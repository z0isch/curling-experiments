mod hex_grid;
mod intersection;
mod tile;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};
use hex_grid::{HexCoordinate, HexGrid, hex_grid, hex_to_world, world_to_hex};
use tile::{
    TileAssets, TileFill, TileType, change_tile_type, compute_tile_effects, tile,
    toggle_tile_coordinates, update_tile_hover_material,
};

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
        .add_systems(Update, change_tile_type)
        .add_systems(Update, draw_move_line)
        .add_systems(Update, restart_game)
        .run();
}

#[derive(Resource, Clone)]
struct UiState {
    drag_coefficient: f32,
    stone_velocity_x: f32,
    stone_velocity_y: f32,
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

#[derive(Component, Clone)]
struct Stone;

#[derive(Component)]
struct StoneMoveLine;

#[derive(Component, Clone)]
pub struct Velocity(pub Vec2);

const STONE_RADIUS: f32 = 10.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let grid = HexGrid::new(35.0, 15, 10);

    let initial_velocity = Vec2::from_angle(-std::f32::consts::FRAC_PI_3 / 2.) * 300.0;
    let ui_state = UiState {
        drag_coefficient: 0.002,
        stone_velocity_x: initial_velocity.x,
        stone_velocity_y: initial_velocity.y,
    };
    let stone_velocity_x = ui_state.stone_velocity_x;
    let stone_velocity_y = ui_state.stone_velocity_y;
    commands.insert_resource(ui_state);

    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);

    commands.spawn(Camera2d);

    let hex_border_mesh = meshes.add(RegularPolygon::new(grid.hex_radius, 6));
    let black_material = materials.add(Color::BLACK);

    // Build tile entities to spawn as children
    let mut tile_entities: Vec<Entity> = Vec::new();

    for q in 0..grid.cols {
        for r in 0..grid.rows {
            let world_pos = hex_to_world(&HexCoordinate { q, r }, &grid);
            let tile_type = if q == 0 || q == grid.cols - 1 || r == 0 || r == grid.rows - 1 {
                TileType::Wall
            } else if q == 8 && r == 4 {
                TileType::Goal
            } else {
                TileType::SlowDown
            };

            let tile_entity =
                commands
                    .spawn(tile(
                        tile_type,
                        world_pos,
                        q,
                        r,
                        hex_border_mesh.clone(),
                        black_material.clone(),
                        &tile_assets,
                    ))
                    .observe(
                        |click: On<Pointer<Click>>,
                         camera: Single<(&Camera, &GlobalTransform)>,
                         grid: Single<&HexGrid>,
                         mut stone: Single<&mut Transform, With<Stone>>| {
                            let Ok(world_pos) = camera
                                .0
                                .viewport_to_world_2d(camera.1, click.pointer_location.position)
                            else {
                                return;
                            };
                            let Some(hex_coord) = world_to_hex(world_pos, *grid) else {
                                return;
                            };
                            stone.translation = hex_to_world(&hex_coord, *grid).extend(3.0);
                        },
                    )
                    .observe(
                        |over: On<Pointer<Over>>,
                         tile_type: Query<&TileType>,
                         children: Query<&Children>,
                         tile_assets: Res<TileAssets>,
                         mut fill_query: Query<
                            &mut MeshMaterial2d<ColorMaterial>,
                            With<TileFill>,
                        >| {
                            update_tile_hover_material(
                                over.entity,
                                true,
                                &tile_type,
                                &children,
                                &tile_assets,
                                &mut fill_query,
                            );
                        },
                    )
                    .observe(
                        |out: On<Pointer<Out>>,
                         tile_type: Query<&TileType>,
                         children: Query<&Children>,
                         tile_assets: Res<TileAssets>,
                         mut fill_query: Query<
                            &mut MeshMaterial2d<ColorMaterial>,
                            With<TileFill>,
                        >| {
                            update_tile_hover_material(
                                out.entity,
                                false,
                                &tile_type,
                                &children,
                                &tile_assets,
                                &mut fill_query,
                            );
                        },
                    )
                    .id();
            tile_entities.push(tile_entity);
        }
    }

    commands.insert_resource(tile_assets);

    // Spawn HexGrid entity and add tiles as children
    commands
        .spawn(hex_grid(35.0, 15, 10))
        .add_children(&tile_entities);

    let stone_mesh = meshes.add(Circle::new(STONE_RADIUS));
    let stone_world_pos = hex_to_world(&HexCoordinate { q: 1, r: 1 }, &grid);
    commands.spawn((
        Stone,
        Velocity(Vec2::new(stone_velocity_x, stone_velocity_y)),
        Mesh2d(stone_mesh),
        MeshMaterial2d(black_material),
        Transform::from_xyz(stone_world_pos.x, stone_world_pos.y, 3.0),
    ));
}

/// System that restarts the game when 'R' key is pressed
fn restart_game(
    input: Res<ButtonInput<KeyCode>>,
    grid: Single<&HexGrid>,
    ui_state: Res<UiState>,
    mut stone: Single<(&mut Velocity, &mut Transform), With<Stone>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        let initial_hex = HexCoordinate { q: 1, r: 1 };
        let stone_world_pos = hex_to_world(&initial_hex, *grid);
        stone.0.0 = Vec2::new(ui_state.stone_velocity_x, ui_state.stone_velocity_y);
        stone.1.translation = stone_world_pos.extend(3.0);
    }
}

fn update_stone_position(
    mut stone: Single<(&Velocity, &mut Transform), With<Stone>>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    time: Res<Time>,
    grid: Single<&HexGrid>,
) {
    //If velocity is zero and on the goal tile, center it in the hex
    if stone.0.0.length_squared() <= 1.
        && tiles.iter().any(|(tile_type, transform)| {
            tile_type == &TileType::Goal
                && intersection::circle_hexagon_overlap_ratio(
                    stone.1.translation.truncate(),
                    STONE_RADIUS,
                    transform.translation.truncate(),
                    grid.hex_radius,
                    100,
                ) >= 0.9
        })
    {
        stone.1.translation = hex_to_world(&HexCoordinate { q: 8, r: 4 }, *grid).extend(3.0);
    } else {
        let delta = stone.0.0 * time.delta_secs();
        stone.1.translation += delta.extend(0.);
    }
}

/// System that modifies stone velocity based on tile types it overlaps with.
/// Uses circle_hexagon_overlap_ratio as a multiplicative factor for the effect strength.
fn apply_tile_velocity_effects(
    mut stone: Single<(&mut Velocity, &mut Transform), With<Stone>>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    grid: Single<&HexGrid>,
    ui_state: Res<UiState>,
) {
    const SAMPLES: u32 = 100;
    let tile_data: Vec<_> = tiles
        .iter()
        .map(|(tile_type, transform)| (tile_type, transform.translation.truncate()))
        .collect();
    *stone.0 = compute_tile_effects(
        stone.1.translation.truncate(),
        &stone.0,
        &tile_data,
        *grid,
        ui_state.drag_coefficient,
        SAMPLES,
        STONE_RADIUS,
    );
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
    const DT: f32 = 1.0 / 60.0; // Simulate at 644fps
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
