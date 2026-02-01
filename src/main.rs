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
use stone::{
    STONE_RADIUS, Stone, Velocity, apply_tile_velocity_effects, restart_game, stone,
    update_stone_position,
};
use tile::{TileAssets, TileType, change_tile_type, compute_tile_effects, toggle_tile_coordinates};

#[derive(Component)]
struct StoneMoveLine;

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
            (update_stone_position, apply_tile_velocity_effects).chain(),
        )
        .add_systems(Update, toggle_tile_coordinates)
        .add_systems(Update, (change_tile_type, draw_move_line, restart_game))
        .run();
}

#[derive(Resource, Clone)]
pub struct UiState {
    pub drag_coefficient: f32,
    pub stone_velocity_x: f32,
    pub stone_velocity_y: f32,
}

fn ui(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) -> Result {
    egui::Window::new("").show(contexts.ctx_mut()?, |ui| {
        ui.add(
            egui::Slider::new(&mut ui_state.drag_coefficient, 0.001..=0.05)
                .text("Drag Coefficient"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.stone_velocity_x, -500.0..=500.0)
                .text("Stone Velocity X"),
        );
        ui.add(
            egui::Slider::new(&mut ui_state.stone_velocity_y, -500.0..=500.0)
                .text("Stone Velocity Y"),
        );
    });
    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let initial_velocity = Vec2::from_angle(-std::f32::consts::FRAC_PI_3 / 2.) * 300.0;
    let ui_state = UiState {
        drag_coefficient: 0.002,
        stone_velocity_x: initial_velocity.x,
        stone_velocity_y: initial_velocity.y,
    };
    let stone_velocity_x = ui_state.stone_velocity_x;
    let stone_velocity_y = ui_state.stone_velocity_y;
    commands.insert_resource(ui_state);

    commands.spawn(Camera2d);

    let grid = HexGrid::new(35.0, 15, 10);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);

    spawn_hex_grid(&mut commands, &grid, &tile_assets);

    commands.spawn(stone(
        &mut meshes,
        &mut materials,
        &grid,
        HexCoordinate { q: 1, r: 1 },
        Vec2::new(stone_velocity_x, stone_velocity_y),
    ));

    commands.insert_resource(tile_assets);
}

fn draw_move_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: Res<TileAssets>,
    grid: Single<&HexGrid>,
    ui_state: Res<UiState>,
    stone: Single<(&Velocity, &Transform), With<Stone>>,
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
        &stone.1.translation.truncate(),
        stone.0,
        &tile_data,
        *grid,
        ui_state.drag_coefficient,
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
            STONE_RADIUS,
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
