mod debug_ui;
mod fire_trail;
mod hex_grid;
mod intersection;
mod level;
mod stone;
mod tile;
mod ui;

use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use bevy::window::WindowResolution;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};
use debug_ui::{DebugUIState, StoneUIConfig, debug_ui};
use fire_trail::{spawn_fire_trail, update_fire_trail};
use hex_grid::{HexGrid, spawn_hex_grid};
use stone::{
    Stone, Velocity, apply_tile_velocity_effects, resolve_collision, stone, update_stone_position,
};
use tile::{
    CurrentDragTileType, ScratchOffMaterial, TileAssets, TileDragging, TileType,
    compute_tile_effects, toggle_tile_coordinates,
};

use crate::{
    debug_ui::on_debug_ui_level_change,
    level::{CurrentLevel, get_initial_stone_velocity, get_level},
    stone::apply_stone_collision,
    tile::{MouseHover, update_tile_material, update_tile_type},
    ui::{
        CountdownUI, spawn_broom_type_ui, spawn_countdown, update_broom_type_ui, update_countdown,
    },
};

#[derive(Component)]
struct StoneMoveLine;
#[derive(Resource)]
pub struct Countdown {
    pub timer: Timer,
    pub count: u32,
}

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
        .add_plugins(Material2dPlugin::<ScratchOffMaterial>::default())
        .add_plugins((
            EntropyPlugin::<ChaCha8Rng>::default(),
            EntropyPlugin::<WyRand>::default(),
        ))
        .add_systems(EguiPrimaryContextPass, debug_ui)
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
            (spawn_fire_trail, update_fire_trail).run_if(|paused: Res<PhysicsPaused>| !paused.0),
        )
        .add_systems(
            Update,
            (
                draw_move_line,
                update_tile_type,
                toggle_physics_pause,
                toggle_tile_coordinates,
                update_tile_material,
                update_countdown,
                switch_broom,
                update_broom_type_ui,
                drag_with_keyboard,
            )
                .in_set(MainUpdateSystems),
        )
        .add_systems(
            Update,
            (restart_game_on_r_key_pressed, on_debug_ui_level_change).after(MainUpdateSystems),
        )
        .run();
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainUpdateSystems;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
) {
    commands.insert_resource(PhysicsPaused(true));
    let current_level = CurrentLevel::default();
    let level = get_level(current_level);
    let debug_ui_state = DebugUIState {
        hex_radius: 60.0,
        stone_radius: 15.0,
        min_sweep_distance: 250.0,
        drag_coefficient: 0.0036,
        slow_down_factor: 5.0,
        rotation_factor: 0.025,
        snap_distance: 40.0,
        snap_velocity: 40.0,
        current_level,
        stone_configs: level
            .stone_configs
            .iter()
            .map(|sc| StoneUIConfig {
                velocity_magnitude: sc.velocity_magnitude,
                facing: sc.facing.clone(),
            })
            .collect(),
    };

    commands.spawn(Camera2d);

    let grid = HexGrid::new(debug_ui_state.hex_radius, &level);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);

    spawn_hex_grid(&mut commands, &grid, &tile_assets, &mut scratch_materials);
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
    commands.insert_resource(CurrentDragTileType(TileType::MaintainSpeed));

    spawn_broom_type_ui(&mut commands);
    spawn_countdown(&mut commands, level.countdown);
    commands.insert_resource(Countdown {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        count: level.countdown,
    });
}

/// System that toggles physics pause when Space is pressed
fn toggle_physics_pause(input: Res<ButtonInput<KeyCode>>, mut paused: ResMut<PhysicsPaused>) {
    if input.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
}

pub fn switch_broom(
    input: Res<ButtonInput<KeyCode>>,
    mut current_drag_tile_type: ResMut<CurrentDragTileType>,
) {
    if input.just_pressed(KeyCode::Digit1) {
        *current_drag_tile_type = CurrentDragTileType(TileType::MaintainSpeed);
    }
    if input.just_pressed(KeyCode::Digit2) {
        *current_drag_tile_type = CurrentDragTileType(TileType::TurnCounterclockwise);
    }
    if input.just_pressed(KeyCode::Digit3) {
        *current_drag_tile_type = CurrentDragTileType(TileType::TurnClockwise);
    }
}

pub fn restart_game_on_r_key_pressed(
    input: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    grid: Single<Entity, With<HexGrid>>,
    countdown_ui_query: Query<Entity, With<CountdownUI>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    paused: ResMut<PhysicsPaused>,
    countdown: ResMut<Countdown>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        restart_game(
            commands,
            grid,
            countdown_ui_query,
            debug_ui_state,
            stone_query,
            paused,
            countdown,
            meshes,
            materials,
            scratch_materials,
        );
    }
}
/// System that restarts the game when 'R' key is pressed
pub fn restart_game(
    mut commands: Commands,
    grid: Single<Entity, With<HexGrid>>,
    countdown_ui_query: Query<Entity, With<CountdownUI>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    mut paused: ResMut<PhysicsPaused>,
    mut countdown: ResMut<Countdown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
) {
    paused.0 = true;
    let level = get_level(debug_ui_state.current_level);

    commands.entity(*grid).despawn();

    let grid = HexGrid::new(debug_ui_state.hex_radius, &level);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);
    spawn_hex_grid(&mut commands, &grid, &tile_assets, &mut scratch_materials);
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
    for entity in countdown_ui_query {
        commands.entity(entity).despawn();
    }
    countdown.count = level.countdown;
    countdown.timer.reset();
    spawn_countdown(&mut commands, level.countdown);
}

fn draw_move_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: Res<TileAssets>,
    grid: Single<&HexGrid>,
    debug_ui_state: Res<DebugUIState>,
    stones: Query<(&Stone, &Velocity, &Transform)>,
    tiles: Query<(&TileType, &Transform, Option<&TileDragging>), Without<Stone>>,
    lines: Query<Entity, With<StoneMoveLine>>,
    fixed_time: Res<Time<Fixed>>,
) {
    for l in &lines {
        commands.entity(l).despawn();
    }

    // Collect tile data for trajectory simulation (including dragging state)
    let tile_data: Vec<_> = tiles
        .iter()
        .map(|(tile_type, transform, tile_dragging)| {
            let position = transform.translation.truncate();
            (tile_type, position, tile_dragging)
        })
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
        debug_ui_state.min_sweep_distance,
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
    tile_data: &[(&TileType, Vec2, Option<&TileDragging>)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    fixed_dt: f32,
    slow_down_factor: f32,
    rotation_factor: f32,
    min_sweep_distance: f32,
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
                min_sweep_distance,
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

fn drag_with_keyboard(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    current_drag_tile_type: Res<CurrentDragTileType>,
    tile_query: Single<(Entity, Option<&mut TileDragging>), With<MouseHover>>,
) {
    if let Some(just_pressed) = input.get_just_pressed().next() {
        let (entity, tile_dragging_opt) = tile_query.into_inner();

        match tile_dragging_opt {
            Some(mut tile_dragging) => {
                if let Some(last_key_pressed) =
                    tile_dragging.last_keyboard_input.replace(*just_pressed)
                    && last_key_pressed != *just_pressed
                {
                    tile_dragging.distance_dragged += 50.;
                }
            }
            None => {
                // Insert new TileDragging component
                commands.entity(entity).insert(TileDragging {
                    last_position: None,
                    distance_dragged: 0.0,
                    tile_type: current_drag_tile_type.0.clone(),
                    last_keyboard_input: Some(*just_pressed),
                });
            }
        }
    }
}
