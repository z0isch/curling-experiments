mod hexgrid;
mod intersection;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};
use hexgrid::{HexCoordinate, HexGridConfig, hex_to_world, world_to_hex};

fn main() {
    let config = HexGridConfig::new(35.0, 15, 10);

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
        .insert_resource(config)
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

#[derive(Resource)]
struct TileAssets {
    hex_mesh: Handle<Mesh>,
    line_material: Handle<ColorMaterial>,
    wall: TileTypeAssets,
    maintain_speed: TileTypeAssets,
    slow_down: TileTypeAssets,
    turn_counterclockwise: TileTypeAssets,
    turn_clockwise: TileTypeAssets,
    goal: TileTypeAssets,
}

fn get_tile_type_assets<'a>(
    tile_type: &TileType,
    tile_assets: &'a TileAssets,
) -> &'a TileTypeAssets {
    match tile_type {
        TileType::Wall => &tile_assets.wall,
        TileType::MaintainSpeed => &tile_assets.maintain_speed,
        TileType::SlowDown => &tile_assets.slow_down,
        TileType::TurnCounterclockwise => &tile_assets.turn_counterclockwise,
        TileType::TurnClockwise => &tile_assets.turn_clockwise,
        TileType::Goal => &tile_assets.goal,
    }
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
struct TileTypeAssets {
    material: Handle<ColorMaterial>,
    hover_material: Handle<ColorMaterial>,
}

#[derive(Component, PartialEq, Debug)]
enum TileType {
    Wall,
    MaintainSpeed,
    SlowDown,
    TurnCounterclockwise,
    TurnClockwise,
    Goal,
}

#[derive(Component)]
struct TileFill;

#[derive(Component)]
struct TileCoordinateText;

const COLORS: [Color; 6] = [
    // #dcf3ff
    Color::srgb(220.0 / 255.0, 243.0 / 255.0, 1.),
    // #baf2ef
    Color::srgb(186.0 / 255.0, 242.0 / 255.0, 239.0 / 255.0),
    // #a2d2df
    Color::srgb(162.0 / 255.0, 210.0 / 255.0, 223.0 / 255.0),
    // #396d7c
    Color::srgb(57.0 / 255.0, 109.0 / 255.0, 124.0 / 255.0),
    // #257ca3
    Color::srgb(37.0 / 255.0, 124.0 / 255.0, 163.0 / 255.0),
    //rgb(245, 92, 92)
    Color::srgb(245.0 / 255.0, 92.0 / 255.0, 92.0 / 255.0),
];

#[derive(Component, Clone)]
struct Stone;

#[derive(Component)]
struct StoneMoveLine;

#[derive(Component, Clone)]
struct Velocity(Vec2);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HexGridConfig>,
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
    let border_thickness = 1.0;
    let tile_assets = TileAssets {
        hex_mesh: meshes.add(RegularPolygon::new(config.hex_radius - border_thickness, 6)),
        line_material: materials.add(COLORS[5]),
        wall: TileTypeAssets {
            material: materials.add(COLORS[3]),
            hover_material: materials.add(COLORS[3].with_alpha(0.8)),
        },
        maintain_speed: TileTypeAssets {
            material: materials.add(COLORS[0]),
            hover_material: materials.add(COLORS[0].with_alpha(0.8)),
        },
        slow_down: TileTypeAssets {
            material: materials.add(COLORS[1]),
            hover_material: materials.add(COLORS[1].with_alpha(0.8)),
        },
        turn_counterclockwise: TileTypeAssets {
            material: materials.add(COLORS[2]),
            hover_material: materials.add(COLORS[2].with_alpha(0.8)),
        },
        turn_clockwise: TileTypeAssets {
            material: materials.add(COLORS[4]),
            hover_material: materials.add(COLORS[4].with_alpha(0.8)),
        },
        goal: TileTypeAssets {
            material: materials.add(COLORS[5]),
            hover_material: materials.add(COLORS[5].with_alpha(0.8)),
        },
    };

    commands.spawn(Camera2d);

    let hex_border_mesh = meshes.add(RegularPolygon::new(config.hex_radius, 6));
    let black_material = materials.add(Color::BLACK);

    for q in 0..config.cols {
        for r in 0..config.rows {
            let world_pos = hex_to_world(&HexCoordinate { q, r }, &config);
            let tile_type = if q == 0 || q == config.cols - 1 || r == 0 || r == config.rows - 1 {
                TileType::Wall
            } else if q == 8 && r == 4 {
                TileType::Goal
            } else {
                TileType::SlowDown
            };

            let assets = get_tile_type_assets(&tile_type, &tile_assets);

            commands
                .spawn((
                    tile_type,
                    Visibility::Visible,
                    Transform::from_xyz(world_pos.x, world_pos.y, 0.0)
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_6)),
                    children![
                        (
                            Mesh2d(hex_border_mesh.clone()),
                            MeshMaterial2d(black_material.clone())
                        ),
                        (
                            TileFill,
                            Mesh2d(tile_assets.hex_mesh.clone()),
                            MeshMaterial2d(assets.material.clone()),
                            Transform::from_xyz(0., 0., 1.0),
                        ),
                        (
                            TileCoordinateText,
                            Visibility::Hidden,
                            Text2d::new(format!("{},{}", q, r)),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            Transform::from_xyz(0., 0., 2.0)
                                .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_6)),
                        )
                    ],
                ))
                .observe(|click: On<Pointer<Click>>, camera: Single<(&Camera, &GlobalTransform)>, config: Res<HexGridConfig>, mut stone: Single<&mut Transform, With<Stone>>| {
                    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, click.pointer_location.position) else {
                        return;
                    };
                    let Some(hex_coord) = world_to_hex(world_pos, &config) else {
                        return;
                    };
                    stone.translation = hex_to_world(&hex_coord, &config).extend(3.0);
                })
                .observe(
                    |over: On<Pointer<Over>>,
                     tile_type: Query<&TileType>,
                     children: Query<&Children>,
                     tile_assets: Res<TileAssets>,
                     mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>| {
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
                     mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>| {
                        update_tile_hover_material(
                            out.entity,
                            false,
                            &tile_type,
                            &children,
                            &tile_assets,
                            &mut fill_query,
                        );
                    },
                );
        }
    }

    commands.insert_resource(tile_assets);

    let stone_mesh = meshes.add(Circle::new(STONE_RADIUS));
    let stone_world_pos = hex_to_world(&HexCoordinate { q: 1, r: 1 }, &config);
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
    config: Res<HexGridConfig>,
    ui_state: Res<UiState>,
    mut stone: Single<(&mut Velocity, &mut Transform), With<Stone>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        let initial_hex = HexCoordinate { q: 1, r: 1 };
        let stone_world_pos = hex_to_world(&initial_hex, &config);
        stone.0.0 = Vec2::new(ui_state.stone_velocity_x, ui_state.stone_velocity_y);
        stone.1.translation = stone_world_pos.extend(3.0);
    }
}

fn change_tile_type(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    config: Res<HexGridConfig>,
    input: Res<ButtonInput<KeyCode>>,
    mut tiles: Query<(&mut TileType, &Transform)>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) else {
        return;
    };
    if let Some(hex_coord) = world_to_hex(world_pos, &config) {
        let Some((mut tile_type, _)) = tiles.iter_mut().find(|(_, transform)| {
            world_to_hex(transform.translation.truncate(), &config).as_ref() == Some(&hex_coord)
        }) else {
            log::error!("Tile not found for stone at position: {:?}", hex_coord);
            return;
        };
        if *tile_type == TileType::Goal {
            return;
        }
        if input.just_pressed(KeyCode::KeyW) {
            *tile_type = TileType::MaintainSpeed;
        }
        if input.just_pressed(KeyCode::KeyA) {
            *tile_type = TileType::TurnClockwise;
        }
        if input.just_pressed(KeyCode::KeyD) {
            *tile_type = TileType::TurnCounterclockwise;
        }
        if input.just_pressed(KeyCode::KeyS) {
            *tile_type = TileType::SlowDown;
        }
    }
}

// On pressing the `~` key, toggle the visibility of the tile coordinates
fn toggle_tile_coordinates(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    tiles: Query<(Entity, &Visibility), With<TileCoordinateText>>,
) {
    if input.just_pressed(KeyCode::Backquote) {
        for (entity, visibility) in tiles {
            commands.entity(entity).remove::<Visibility>();
            if let Visibility::Visible = visibility {
                commands.entity(entity).insert(Visibility::Hidden);
            } else {
                commands.entity(entity).insert(Visibility::Visible);
            }
        }
    }
}

const STONE_RADIUS: f32 = 10.0;

fn update_stone_position(
    mut stone: Single<(&Velocity, &mut Transform), With<Stone>>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    time: Res<Time>,
    config: Res<HexGridConfig>,
) {
    //If velocity is zero and on the goal tile, center it in the hex
    if stone.0.0.length_squared() <= 1.
        && tiles.iter().any(|(tile_type, transform)| {
            tile_type == &TileType::Goal
                && intersection::circle_hexagon_overlap_ratio(
                    stone.1.translation.truncate(),
                    STONE_RADIUS,
                    transform.translation.truncate(),
                    config.hex_radius,
                    100,
                ) >= 0.9
        })
    {
        stone.1.translation = hex_to_world(&HexCoordinate { q: 8, r: 4 }, &config).extend(3.0);
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
    config: Res<HexGridConfig>,
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
        &config,
        ui_state.drag_coefficient,
        SAMPLES,
    );
}

/// Computes the new velocity after applying all tile effects at the given position.
/// This is the core physics logic shared by both real-time simulation and trajectory prediction.
fn compute_tile_effects(
    pos: Vec2,
    velocity: &Velocity,
    tiles: &[(&TileType, Vec2)],
    config: &HexGridConfig,
    drag_coefficient: f32,
    samples: u32,
) -> Velocity {
    let mut rotation_angle = 0.0_f32;
    let mut total_drag = 0.0_f32;
    let mut new_velocity = velocity.clone();

    for &(tile_type, tile_world_pos) in tiles {
        let overlap_ratio = intersection::circle_hexagon_overlap_ratio(
            pos,
            STONE_RADIUS,
            tile_world_pos,
            config.hex_radius,
            samples,
        );

        if overlap_ratio <= 0.0 {
            continue;
        }

        match tile_type {
            TileType::Wall => {
                // Immediately reflect velocity off the wall
                let to_wall = tile_world_pos - pos;
                if to_wall.length_squared() > 0.0 {
                    let wall_normal = -to_wall.normalize();
                    // Reflect: v' = v - 2(v·n)n
                    let dot = new_velocity.0.dot(wall_normal);
                    if dot < 0.0 {
                        // Only reflect if moving toward the wall
                        new_velocity.0 -= 2.0 * dot * wall_normal;
                    }
                }
            }
            TileType::MaintainSpeed => {
                total_drag += drag_coefficient * overlap_ratio;
            }
            TileType::SlowDown => {
                total_drag += drag_coefficient * 2. * overlap_ratio;
            }
            TileType::TurnCounterclockwise => {
                // Rotate velocity counterclockwise, scaled by overlap
                // ~1 degree per frame at full overlap
                rotation_angle += 0.017 * overlap_ratio;
                // Apply drag proportional to overlap
                total_drag += drag_coefficient * overlap_ratio;
            }
            TileType::TurnClockwise => {
                // Rotate velocity clockwise, scaled by overlap
                rotation_angle -= 0.017 * overlap_ratio;
                // Apply drag proportional to overlap
                total_drag += drag_coefficient * overlap_ratio;
            }
            TileType::Goal => {
                total_drag += drag_coefficient * overlap_ratio;
                if overlap_ratio >= 0.9 && new_velocity.0.length_squared() < 20. * 20. {
                    new_velocity.0 = Vec2::ZERO;
                }
            }
        }
    }

    // Apply accumulated rotation to velocity vector
    if rotation_angle.abs() > 0.0001 {
        let cos_angle = rotation_angle.cos();
        let sin_angle = rotation_angle.sin();
        new_velocity.0 = Vec2::new(
            new_velocity.0.x * cos_angle - new_velocity.0.y * sin_angle,
            new_velocity.0.x * sin_angle + new_velocity.0.y * cos_angle,
        );
    }

    // Apply accumulated drag - reduces velocity magnitude while preserving direction
    if total_drag > 0.0 {
        // Clamp drag factor to prevent velocity reversal
        let drag_factor = (1.0 - total_drag).max(0.0);
        new_velocity.0 *= drag_factor;
    }

    new_velocity
}

fn draw_move_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: Res<TileAssets>,
    config: Res<HexGridConfig>,
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
        &config,
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
    config: &HexGridConfig,
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
        velocity =
            compute_tile_effects(pos, &velocity, tile_data, config, drag_coefficient, SAMPLES);

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

/// Updates the tile fill material for hover effects.
fn update_tile_hover_material(
    entity: Entity,
    hovered: bool,
    tile_type_query: &Query<&TileType>,
    children_query: &Query<&Children>,
    tile_assets: &TileAssets,
    fill_query: &mut Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>,
) {
    let Ok(tile_type) = tile_type_query.get(entity) else {
        return;
    };
    if *tile_type == TileType::Wall || *tile_type == TileType::Goal {
        return;
    }
    let Ok(children) = children_query.get(entity) else {
        return;
    };
    let assets = get_tile_type_assets(tile_type, tile_assets);
    let material = if hovered {
        &assets.hover_material
    } else {
        &assets.material
    };
    for child in children.iter() {
        if let Ok(mut mesh_material) = fill_query.get_mut(child) {
            mesh_material.0 = material.clone();
        }
    }
}
