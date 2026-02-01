mod hex_grid;
mod intersection;
mod stone;
mod tile;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};
use hex_grid::{HexCoordinate, HexGrid, spawn_hex_grid};
use stone::{Stone, Velocity, apply_tile_velocity_effects, stone, update_stone_position};
use tile::{TileAssets, TileType, change_tile_type, compute_tile_effects, toggle_tile_coordinates};

use crate::{
    hex_grid::{Facing, get_initial_stone_velocity, get_level, hex_to_world},
    tile::update_sweep_count,
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
            (update_stone_position, apply_tile_velocity_effects)
                .chain()
                .run_if(|paused: Res<PhysicsPaused>| !paused.0),
        )
        .add_systems(Update, toggle_tile_coordinates)
        .add_systems(
            Update,
            (
                change_tile_type,
                draw_move_line,
                restart_game,
                update_sweep_count,
                toggle_physics_pause,
            ),
        )
        .run();
}

#[derive(Resource, Clone, Debug)]
pub struct UiState {
    pub drag_coefficient: f32,
    pub stone_velocity_magnitude: f32,
    pub stone_facing: Facing,
    pub min_sweep_distance: f32,
    pub hex_radius: f32,
    pub stone_radius: f32,
}

fn ui(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) -> Result {
    egui::Window::new("").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::Label::new("R to restart"));
        ui.add(egui::Label::new("Space to pause/resume"));
        ui.add(egui::Slider::new(&mut ui_state.hex_radius, 20.0..=80.0).text("Hex Radius"));
        ui.add(egui::Slider::new(&mut ui_state.stone_radius, 10.0..=30.0).text("Stone Radius"));
        ui.add(
            egui::Slider::new(&mut ui_state.min_sweep_distance, 0.0..=200.0)
                .text("Min Sweep Distance"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.drag_coefficient, 0.001..=0.01)
                .text("Drag Coefficient"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.stone_velocity_magnitude, 0.0..=500.0)
                .text("Stone Velocity Magnitude"),
        );
        egui::ComboBox::from_label("Stone Facing")
            .selected_text(format!("{:?}", ui_state.stone_facing))
            .show_ui(ui, |ui| {
                for facing in Facing::iterator() {
                    ui.selectable_value(
                        &mut ui_state.stone_facing,
                        facing.clone(),
                        facing.to_string(),
                    );
                }
            });
    });
    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(PhysicsPaused(true));
    let level = get_level();
    let initial_velocity =
        get_initial_stone_velocity(&level.facing, &level.stone_velocity_magnitude);
    let ui_state = UiState {
        drag_coefficient: 0.01,
        stone_velocity_magnitude: level.stone_velocity_magnitude,
        stone_facing: level.facing.clone(),
        min_sweep_distance: 175.0,
        hex_radius: 70.0,
        stone_radius: 25.0,
    };

    commands.spawn(Camera2d);

    let grid = HexGrid::new(ui_state.hex_radius, &level);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);

    spawn_hex_grid(&mut commands, &grid, &tile_assets);

    commands.spawn(stone(
        &mut meshes,
        &mut materials,
        &grid,
        &grid.level.start_coordinate,
        initial_velocity,
        ui_state.stone_radius,
    ));

    commands.insert_resource(tile_assets);
    commands.insert_resource(ui_state);
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
    ui_state: Res<UiState>,
    stone_query: Single<Entity, With<Stone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        let mut level = get_level();
        level.facing = ui_state.stone_facing.clone();
        level.stone_velocity_magnitude = ui_state.stone_velocity_magnitude;

        commands.entity(*grid).despawn();

        let grid = HexGrid::new(ui_state.hex_radius, &level);
        let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);
        spawn_hex_grid(&mut commands, &grid, &tile_assets);
        commands.entity(*stone_query).despawn();
        commands.spawn(stone(
            &mut meshes,
            &mut materials,
            &grid,
            &level.start_coordinate,
            get_initial_stone_velocity(&level.facing, &level.stone_velocity_magnitude),
            ui_state.stone_radius,
        ));
    }
}

fn draw_move_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: Res<TileAssets>,
    grid: Single<&HexGrid>,
    ui_state: Res<UiState>,
    stone: Single<(&Stone, &Velocity, &Transform)>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    lines: Query<Entity, With<StoneMoveLine>>,
) {
    for l in &lines {
        commands.entity(l).despawn();
    }

    // Collect tile data for trajectory simulation
    let tile_data: Vec<_> = tiles
        .iter()
        .map(|(tile_type, transform)| (tile_type, transform.translation.truncate()))
        .collect();

    // Simulate physics forward to predict trajectory
    let trajectory = simulate_trajectory(
        &stone.2.translation.truncate(),
        stone.1,
        &tile_data,
        *grid,
        ui_state.drag_coefficient,
        stone.0.radius,
    );

    // Draw line segments between trajectory points
    for window in trajectory.windows(2) {
        let (start, end) = (window[0], window[1]);
        commands.spawn((
            StoneMoveLine,
            Mesh2d(meshes.add(Segment2d::new(start, end))),
            MeshMaterial2d(tile_assets.line_material.clone()),
            Transform::from_xyz(0., 0., 3.0),
        ));
    }
}

/// Simulates the stone's trajectory by forward-integrating physics
fn simulate_trajectory(
    position: &Vec2,
    velocity: &Velocity,
    tile_data: &[(&TileType, Vec2)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    stone_radius: f32,
) -> Vec<Vec2> {
    const SAMPLES: u32 = 20; // Fewer samples for performance in prediction
    const DT: f32 = 1.0 / 60.0; // Simulate at 60fps
    const MAX_STEPS: usize = 600; // ~10 seconds of prediction
    const MIN_VELOCITY: f32 = 1.0; // Stop when velocity is very low
    const SAMPLE_INTERVAL: usize = 10; // Only record every Nth position to reduce line segments

    let mut trajectory = vec![*position];
    let mut pos = *position;
    let mut velocity = velocity.clone();

    for step in 0..MAX_STEPS {
        if velocity.0.length_squared() < MIN_VELOCITY * MIN_VELOCITY {
            break;
        }

        // Apply tile effects using shared physics logic
        velocity = compute_tile_effects(
            pos,
            &velocity,
            tile_data,
            hex_grid,
            drag_coefficient,
            SAMPLES,
            stone_radius,
        );

        // Step position forward
        pos += velocity.0 * DT;

        // Only record every Nth position to reduce line segments
        if step % SAMPLE_INTERVAL == 0 {
            trajectory.push(pos);
        }
    }

    // Always include the final position
    if trajectory.last() != Some(&pos) {
        trajectory.push(pos);
    }

    trajectory
}
