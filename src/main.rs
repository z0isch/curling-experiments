use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};

/// Resource containing hexagonal grid parameters
#[derive(Resource)]
struct HexGridConfig {
    hex_radius: f32,
    horiz_spacing: f32,
    vert_spacing: f32,
    cols: i32,
    rows: i32,
    offset_x: f32,
    offset_y: f32,
}

impl HexGridConfig {
    fn new(hex_radius: f32, cols: i32, rows: i32) -> Self {
        let horiz_spacing = hex_radius * 1.5;
        let vert_spacing = hex_radius * 3.0_f32.sqrt();
        let offset_x = -(cols as f32 * horiz_spacing) / 2.0;
        let offset_y = -(rows as f32 * vert_spacing) / 2.0;

        Self {
            hex_radius,
            horiz_spacing,
            vert_spacing,
            cols,
            rows,
            offset_x,
            offset_y,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HexCoordinate {
    pub q: i32,
    pub r: i32,
}

/// Event emitted when the mouse hovers over a tile
#[derive(Event, Debug)]
struct MouseTileHoverEvent(Option<HexCoordinate>);

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
        .add_plugins((
            EntropyPlugin::<ChaCha8Rng>::default(),
            EntropyPlugin::<WyRand>::default(),
        ))
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_tile_coordinates)
        .add_systems(Update, track_mouse_tile)
        .add_systems(Update, draw_stone)
        .add_systems(Update, click_tile)
        .add_systems(Update, move_stone_on_space)
        .add_systems(Update, change_tile_type)
        .add_observer(highlight_tile)
        .run();
}

#[derive(Resource)]
struct TileAssets {
    hex_mesh: Handle<Mesh>,
    wall: TileTypeAssets,
    maintain_speed: TileTypeAssets,
    slow_down: TileTypeAssets,
    turn_counterclockwise: TileTypeAssets,
    turn_clockwise: TileTypeAssets,
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
    }
}

struct TileTypeAssets {
    material: Handle<ColorMaterial>,
    hover_material: Handle<ColorMaterial>,
}

#[derive(Component, PartialEq, Debug)]
struct Tile {
    hex_coord: HexCoordinate,
    tile_type: TileType,
}

#[derive(PartialEq, Debug)]
enum TileType {
    Wall,
    MaintainSpeed,
    SlowDown,
    TurnCounterclockwise,
    TurnClockwise,
}

#[derive(Component)]
struct TileFill;

#[derive(Component)]
struct TileCoordinateText;

const COLORS: [Color; 6] = [
    // #dcf3ff
    Color::srgb(220.0 / 255.0, 243.0 / 255.0, 255.0 / 255.0),
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Facing {
    Up,
    UpRight,
    DownRight,
    Down,
    DownLeft,
    UpLeft,
}

impl Facing {
    /// Returns the (dq, dr) offset for moving in this direction
    /// The offset depends on whether we're in an odd or even column (odd-q offset coordinates)
    fn to_offset(self, in_odd_column: bool) -> (i32, i32) {
        if in_odd_column {
            match self {
                Facing::Up => (0, -1),
                Facing::Down => (0, 1),
                Facing::UpRight => (1, -1),
                Facing::DownRight => (1, 0),
                Facing::DownLeft => (-1, 0),
                Facing::UpLeft => (-1, -1),
            }
        } else {
            match self {
                Facing::Up => (0, -1),
                Facing::Down => (0, 1),
                Facing::UpRight => (1, 0),
                Facing::DownRight => (1, 1),
                Facing::DownLeft => (-1, 1),
                Facing::UpLeft => (-1, 0),
            }
        }
    }

    fn rotate_counterclockwise(self) -> Self {
        match self {
            Facing::Up => Facing::UpLeft,
            Facing::UpLeft => Facing::DownLeft,
            Facing::DownLeft => Facing::Down,
            Facing::Down => Facing::DownRight,
            Facing::DownRight => Facing::UpRight,
            Facing::UpRight => Facing::Up,
        }
    }
    /// Rotate clockwise to the next direction
    fn rotate_clockwise(self) -> Self {
        match self {
            Facing::Up => Facing::UpRight,
            Facing::UpRight => Facing::DownRight,
            Facing::DownRight => Facing::Down,
            Facing::Down => Facing::DownLeft,
            Facing::DownLeft => Facing::UpLeft,
            Facing::UpLeft => Facing::Up,
        }
    }

    /// Convert facing direction to rotation angle in radians
    /// The angle is relative to an arrow pointing up (+y direction)
    fn to_angle(self) -> f32 {
        match self {
            Facing::Up => 0.0,
            Facing::UpRight => -std::f32::consts::FRAC_PI_3, // -60°
            Facing::DownRight => -2.0 * std::f32::consts::FRAC_PI_3, // -120°
            Facing::Down => std::f32::consts::PI,            // 180°
            Facing::DownLeft => 2.0 * std::f32::consts::FRAC_PI_3, // 120°
            Facing::UpLeft => std::f32::consts::FRAC_PI_3,   // 60°
        }
    }
}

#[derive(Component)]
struct Stone {
    pos: HexCoordinate,
    facing: Facing,
    speed: i32,
}

#[derive(Component)]
struct StoneArrow;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HexGridConfig>,
) {
    let border_thickness = 1.0;
    let tile_assets = TileAssets {
        hex_mesh: meshes.add(RegularPolygon::new(config.hex_radius - border_thickness, 6)),
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
    };

    commands.spawn(Camera2d::default());

    let hex_border_mesh = meshes.add(RegularPolygon::new(config.hex_radius, 6));
    let black_material = materials.add(Color::BLACK);

    for q in 0..config.cols {
        for r in 0..config.rows {
            let world_pos = hex_to_world(&HexCoordinate { q, r }, &config);
            let tile_type = if q == 0 || q == config.cols - 1 || r == 0 || r == config.rows - 1 {
                TileType::Wall
            } else {
                TileType::SlowDown
            };

            let assets = get_tile_type_assets(&tile_type, &tile_assets);

            commands.spawn((
                Tile {
                    hex_coord: HexCoordinate { q, r },
                    tile_type,
                },
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
            ));
        }
    }

    commands.insert_resource(tile_assets);

    let stone_hex_coord = HexCoordinate { q: 1, r: 1 };
    let stone_world_pos = hex_to_world(&stone_hex_coord, &config);

    // Arrow mesh pointing up (+y direction)
    let arrow_mesh = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 12.0),  // tip pointing up
        Vec2::new(-6.0, -4.0), // lower left corner
        Vec2::new(6.0, -4.0),  // lower right corner
    ));
    let facing = Facing::DownRight;
    let facing_angle = facing.to_angle();

    commands.spawn((
        Stone {
            pos: stone_hex_coord,
            facing,
            speed: 100,
        },
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(black_material.clone()),
        Transform::from_xyz(stone_world_pos.x, stone_world_pos.y, 3.0),
        children![(
            StoneArrow,
            Mesh2d(arrow_mesh),
            MeshMaterial2d(materials.add(COLORS[5])),
            Transform::from_xyz(0., 0., 1.0).with_rotation(Quat::from_rotation_z(facing_angle)),
        )],
    ));
}

fn hex_to_world(hex_coord: &HexCoordinate, config: &HexGridConfig) -> Vec2 {
    let x = config.offset_x + hex_coord.q as f32 * config.horiz_spacing;
    let y_offset = if hex_coord.q % 2 == 1 {
        config.vert_spacing / 2.0
    } else {
        0.0
    };
    let y =
        config.offset_y + (config.rows - 1 - hex_coord.r) as f32 * config.vert_spacing + y_offset;

    Vec2::new(x, y)
}

/// Converts world position to hex grid coordinates for flat-top hexagons
fn world_to_hex(world_pos: Vec2, config: &HexGridConfig) -> Option<HexCoordinate> {
    // Translate position relative to grid origin
    let rel_x = world_pos.x - config.offset_x;
    let rel_y = world_pos.y - config.offset_y;

    // Estimate column (accounting for horizontal spacing)
    let q_estimate = (rel_x / config.horiz_spacing).round() as i32;

    // Check bounds
    if q_estimate < 0 || q_estimate >= config.cols {
        return None;
    }

    // Account for vertical offset on odd columns
    let y_offset = if q_estimate % 2 == 1 {
        config.vert_spacing / 2.0
    } else {
        0.0
    };

    // Estimate row (r=0 at top, inverted from y coordinate)
    let visual_r = ((rel_y - y_offset) / config.vert_spacing).round() as i32;
    let r_estimate = (config.rows - 1) - visual_r;

    // Check bounds
    if r_estimate < 0 || r_estimate >= config.rows {
        return None;
    }

    // Calculate the center of this hex cell (using inverted r for y position)
    let hex_center_x = config.offset_x + q_estimate as f32 * config.horiz_spacing;
    let hex_center_y =
        config.offset_y + (config.rows - 1 - r_estimate) as f32 * config.vert_spacing + y_offset;

    // Check if point is actually within the hexagon (using distance check)
    // For flat-top hexagons, the inner radius (apothem) = radius * sqrt(3)/2
    let dx = (world_pos.x - hex_center_x).abs();
    let dy = (world_pos.y - hex_center_y).abs();

    // Simple bounding check using the hexagon's geometry
    let inner_radius = config.hex_radius * 3.0_f32.sqrt() / 2.0;

    // For a flat-top hexagon, check if point is inside
    // Using the hex boundary equations
    if dx > config.hex_radius || dy > inner_radius {
        return None;
    }

    // More precise check for the angled edges
    // For flat-top hex: the slanted edges have slope related to the hex geometry
    if dx * inner_radius + dy * config.hex_radius / 2.0 > config.hex_radius * inner_radius {
        return None;
    }

    Some(HexCoordinate {
        q: q_estimate,
        r: r_estimate,
    })
}

/// System that tracks mouse position and emits MouseTileHoverEvent
fn track_mouse_tile(
    mut commands: Commands,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    config: Res<HexGridConfig>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) else {
        return;
    };

    if let Some(hex_coord) = world_to_hex(world_pos, &config) {
        commands.trigger(MouseTileHoverEvent(Some(hex_coord)));
    } else {
        commands.trigger(MouseTileHoverEvent(None));
    }
}

fn highlight_tile(
    mouse_tile_hover_event: On<MouseTileHoverEvent>,
    tile_assets: Res<TileAssets>,
    tiles: Query<(&Tile, &Children)>,
    mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>,
) {
    for (tile, children) in &tiles {
        for child in children.iter() {
            let assets = get_tile_type_assets(&tile.tile_type, &tile_assets);

            if let Ok(mut mesh_material) = fill_query.get_mut(child) {
                if let Some(hex_coord) = &mouse_tile_hover_event.0
                    && tile.hex_coord == *hex_coord
                    && tile.tile_type != TileType::Wall
                {
                    mesh_material.0 = assets.hover_material.clone();
                } else {
                    mesh_material.0 = assets.material.clone();
                }
            }
        }
    }
}

fn change_tile_type(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    config: Res<HexGridConfig>,
    input: Res<ButtonInput<KeyCode>>,
    mut tiles: Query<&mut Tile>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) else {
        return;
    };
    if let Some(hex_coord) = world_to_hex(world_pos, &config) {
        let Some(mut current_tile) = tiles.iter_mut().find(|tile| tile.hex_coord == hex_coord)
        else {
            log::error!("Tile not found for stone at position: {:?}", hex_coord);
            return;
        };

        if input.just_pressed(KeyCode::KeyW) {
            current_tile.tile_type = TileType::MaintainSpeed;
        }
        if input.just_pressed(KeyCode::KeyA) {
            current_tile.tile_type = TileType::TurnClockwise;
        }
        if input.just_pressed(KeyCode::KeyD) {
            current_tile.tile_type = TileType::TurnCounterclockwise;
        }
        if input.just_pressed(KeyCode::KeyS) {
            current_tile.tile_type = TileType::SlowDown;
        }
    }
}

fn click_tile(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    config: Res<HexGridConfig>,
    mut stone: Single<&mut Stone>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        stone.facing = stone.facing.rotate_clockwise();
    }

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) else {
        return;
    };

    if let Some(hex_coord) = world_to_hex(world_pos, &config) {
        if mouse.just_pressed(MouseButton::Left) {
            stone.pos = hex_coord;
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

fn draw_stone(
    mut stone: Single<(&Stone, &mut Transform, &Children)>,
    mut arrow_query: Query<&mut Transform, (With<StoneArrow>, Without<Stone>)>,
    config: Res<HexGridConfig>,
) {
    let (stone_data, ref mut stone_transform, children) = *stone;
    stone_transform.translation = hex_to_world(&stone_data.pos, &config).extend(3.);

    let facing_angle = stone_data.facing.to_angle();
    for child in children.iter() {
        if let Ok(mut arrow_transform) = arrow_query.get_mut(child) {
            arrow_transform.rotation = Quat::from_rotation_z(facing_angle);
        }
    }
}

fn move_stone_on_space(
    input: Res<ButtonInput<KeyCode>>,
    stone: Single<&mut Stone>,
    tiles: Query<&Tile>,
) {
    if input.just_pressed(KeyCode::Space) {
        move_stone(stone, tiles);
    }
}

fn move_stone(mut stone: Single<&mut Stone>, tiles: Query<&Tile>) {
    if stone.speed <= 0 {
        return;
    }

    //Find the tile at the stone's position
    let Some(current_tile) = tiles.iter().find(|tile| tile.hex_coord == stone.pos) else {
        log::error!("Tile not found for stone at position: {:?}", stone.pos);
        return;
    };
    log::info!("Current tile: {:?}", current_tile);

    let facing_direction = match current_tile.tile_type {
        TileType::Wall => stone.facing,
        TileType::MaintainSpeed => stone.facing,
        TileType::SlowDown => stone.facing,
        TileType::TurnCounterclockwise => stone.facing.rotate_counterclockwise(),
        TileType::TurnClockwise => stone.facing.rotate_clockwise(),
    };

    stone.facing = facing_direction;

    let (dq, dr) = facing_direction.to_offset(current_tile.hex_coord.q % 2 == 1);

    let next_tile_coord = HexCoordinate {
        q: current_tile.hex_coord.q + dq,
        r: current_tile.hex_coord.r + dr,
    };

    let Some(next_tile) = tiles.iter().find(|tile| tile.hex_coord == next_tile_coord) else {
        log::error!(
            "Tile not found for stone at position: {:?}",
            next_tile_coord
        );
        return;
    };

    log::info!("Next tile: {:?}", next_tile);

    stone.speed = match current_tile.tile_type {
        TileType::Wall => stone.speed,
        TileType::MaintainSpeed => stone.speed,
        TileType::SlowDown => stone.speed - 1,
        TileType::TurnCounterclockwise => stone.speed - 1,
        TileType::TurnClockwise => stone.speed - 1,
    };

    match next_tile.tile_type {
        TileType::Wall => {
            stone.facing = stone
                .facing
                .rotate_counterclockwise()
                .rotate_counterclockwise()
                .rotate_counterclockwise();
            stone.speed -= 1;
        }
        TileType::MaintainSpeed => {
            stone.pos = next_tile_coord;
        }
        TileType::SlowDown => {
            stone.pos = next_tile_coord;
            stone.speed -= 1;
        }
        TileType::TurnCounterclockwise => {
            stone.pos = next_tile_coord;
            stone.speed -= 1;
        }
        TileType::TurnClockwise => {
            stone.pos = next_tile_coord;
            stone.speed -= 1;
        }
    }
}
