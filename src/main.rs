mod debug_ui;
mod hex_grid;
mod intersection;
mod stone;
mod tile;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};
use debug_ui::{DebugUIState, StoneUIConfig, ui};
use hex_grid::{HexGrid, spawn_hex_grid};
use stone::{
    Stone, Velocity, apply_tile_velocity_effects, resolve_collision, stone, update_stone_position,
};
use tile::{TileAssets, TileType, change_tile_type, compute_tile_effects, toggle_tile_coordinates};

use crate::{
    hex_grid::{get_initial_stone_velocity, get_level},
    stone::apply_stone_collision,
    tile::{update_tile_material, update_tile_type},
};

#[derive(Component)]
struct StoneMoveLine;

#[derive(Resource, Default)]
pub struct PhysicsPaused(pub bool);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1024, 768),
                resizable: false,
                title: "Hexagon Grid".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            EntropyPlugin::<ChaCha8Rng>::default(),
            EntropyPlugin::<WyRand>::default(),
        ))
        .add_systems(EguiPrimaryContextPass, ui)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                apply_stone_collision,
                update_stone_position,
                apply_tile_velocity_effects,
            )
                .chain()
                .run_if(|paused: Res<PhysicsPaused>| !paused.0),
        )
        .add_systems(
            Update,
            (
                change_tile_type,
                draw_move_line,
                restart_game,
                update_tile_type,
                toggle_physics_pause,
                toggle_tile_coordinates,
                update_tile_material,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(PhysicsPaused(true));
    let level = get_level();
    let debug_ui_state = DebugUIState {
        drag_coefficient: 0.0005,
        stone_configs: level
            .stone_configs
            .iter()
            .map(|sc| StoneUIConfig {
                velocity_magnitude: sc.velocity_magnitude,
                facing: sc.facing.clone(),
            })
            .collect(),
        min_sweep_distance: 2.0,
        hex_radius: 20.0,
        stone_radius: 10.0,
        slow_down_factor: 100.0,
        rotation_factor: 0.017,
    };

    commands.spawn(Camera2d);

    let grid = HexGrid::new(debug_ui_state.hex_radius, &level);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);

    spawn_hex_grid(&mut commands, &grid, &tile_assets);
    for stone_config in level.stone_configs {
        commands.spawn(stone(
            &mut meshes,
            &mut materials,
            &grid,
            &stone_config.start_coordinate,
            get_initial_stone_velocity(&stone_config.facing, &stone_config.velocity_magnitude),
            debug_ui_state.stone_radius,
        ));
    }

    commands.insert_resource(tile_assets);
    commands.insert_resource(debug_ui_state);
}

/// System that toggles physics pause when Space is pressed
fn toggle_physics_pause(input: Res<ButtonInput<KeyCode>>, mut paused: ResMut<PhysicsPaused>) {
    if input.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
}

/// System that restarts the game when 'R' key is pressed
pub fn restart_game(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    grid: Single<Entity, With<HexGrid>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        let level = get_level();

        commands.entity(*grid).despawn();

        let grid = HexGrid::new(debug_ui_state.hex_radius, &level);
        let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);
        spawn_hex_grid(&mut commands, &grid, &tile_assets);
        for stone_entity in stone_query {
            commands.entity(stone_entity).despawn();
        }
        for (i, stone_config) in level.stone_configs.iter().enumerate() {
            // Use UI config values if available, otherwise fall back to level defaults
            let (facing, velocity_magnitude) =
                if let Some(ui_config) = debug_ui_state.stone_configs.get(i) {
                    (ui_config.facing.clone(), ui_config.velocity_magnitude)
                } else {
                    (stone_config.facing.clone(), stone_config.velocity_magnitude)
                };
            commands.spawn(stone(
                &mut meshes,
                &mut materials,
                &grid,
                &stone_config.start_coordinate,
                get_initial_stone_velocity(&facing, &velocity_magnitude),
                debug_ui_state.stone_radius,
            ));
        }
    }
}

fn draw_move_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: Res<TileAssets>,
    grid: Single<&HexGrid>,
    debug_ui_state: Res<DebugUIState>,
    stones: Query<(&Stone, &Velocity, &Transform)>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    lines: Query<Entity, With<StoneMoveLine>>,
    fixed_time: Res<Time<Fixed>>,
) {
    for l in &lines {
        commands.entity(l).despawn();
    }

    // Collect tile data for trajectory simulation
    let tile_data: Vec<_> = tiles
        .iter()
        .map(|(tile_type, transform)| (tile_type, transform.translation.truncate()))
        .collect();

    // Collect all stone data for multi-stone simulation
    let stone_data: Vec<_> = stones
        .iter()
        .map(|(stone, velocity, transform)| {
            (
                transform.translation.truncate(),
                velocity.clone(),
                stone.radius,
            )
        })
        .collect();

    // Simulate physics forward to predict trajectory for all stones together
    let trajectories = simulate_trajectories(
        &stone_data,
        &tile_data,
        *grid,
        debug_ui_state.drag_coefficient,
        fixed_time.delta_secs(),
        debug_ui_state.slow_down_factor,
        debug_ui_state.rotation_factor,
    );

    for trajectory in trajectories {
        commands.spawn((
            StoneMoveLine,
            Mesh2d(meshes.add(Polyline2d::new(trajectory))),
            MeshMaterial2d(tile_assets.line_material.clone()),
            Transform::from_xyz(0., 0., 3.0),
        ));
    }
}

/// Simulates all stones' trajectories by forward-integrating physics.
///
/// **Important**: The order of operations must match the FixedUpdate system chain:
/// 1. apply_stone_collision (handle collisions)
/// 2. update_stone_position (move)
/// 3. apply_tile_velocity_effects (update velocity)
fn simulate_trajectories(
    stone_data: &[(Vec2, Velocity, f32)], // (position, velocity, radius)
    tile_data: &[(&TileType, Vec2)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    fixed_dt: f32,
    slow_down_factor: f32,
    rotation_factor: f32,
) -> Vec<Vec<Vec2>> {
    const MIN_VELOCITY: f32 = 1.0; // Stop when velocity is very low
    const LINE_SEGMENT_SAMPLES: usize = 3;

    // Initialize simulation state for each stone
    let mut stones: Vec<_> = stone_data
        .iter()
        .map(|(pos, vel, radius)| (*pos, vel.clone(), *radius))
        .collect();

    let mut trajectories: Vec<Vec<Vec2>> = stones.iter().map(|(pos, _, _)| vec![*pos]).collect();

    let steps = 10000;
    for i in 0..steps {
        // Check if all stones have stopped
        let all_stopped = stones
            .iter()
            .all(|(_, vel, _)| vel.0.length_squared() < MIN_VELOCITY * MIN_VELOCITY);
        if all_stopped {
            break;
        }

        // Step 1: Apply stone collisions (matches apply_stone_collision)
        for j in 0..stones.len() {
            for k in (j + 1)..stones.len() {
                let (pos1, vel1, radius1) = &stones[j];
                let (pos2, vel2, radius2) = &stones[k];

                if let Some((new_vel1, new_vel2)) =
                    resolve_collision(*pos1, vel1, *radius1, *pos2, vel2, *radius2)
                {
                    stones[j].1 = new_vel1;
                    stones[k].1 = new_vel2;
                }
            }
        }

        // Step 2: Move positions (matches update_stone_position)
        for (pos, vel, _) in &mut stones {
            *pos += vel.0 * fixed_dt;
        }

        // Record trajectory points
        if i % LINE_SEGMENT_SAMPLES == 0 {
            for (idx, (pos, _, _)) in stones.iter().enumerate() {
                trajectories[idx].push(*pos);
            }
        }

        // Step 3: Update velocities based on new positions (matches apply_tile_velocity_effects)
        for (pos, vel, radius) in &mut stones {
            *vel = compute_tile_effects(
                *pos,
                vel,
                tile_data,
                hex_grid,
                drag_coefficient,
                *radius,
                slow_down_factor,
                rotation_factor,
            );
        }
    }

    // Always include the final positions
    for (idx, (pos, _, _)) in stones.iter().enumerate() {
        if trajectories[idx].last() != Some(pos) {
            trajectories[idx].push(*pos);
        }
    }

    trajectories
}
