use bevy::prelude::*;

use crate::hex_grid::{HexGrid, world_to_hex};
use crate::intersection;

// ============================================================================
// Bundle Function
// ============================================================================

/// Creates a tile bundle with all visual components for a hexagonal tile.
/// Returns a bundle that can be spawned with `commands.spawn()`.
pub fn tile(
    tile_type: TileType,
    world_pos: Vec2,
    q: i32,
    r: i32,
    hex_border_mesh: Handle<Mesh>,
    black_material: Handle<ColorMaterial>,
    tile_assets: &TileAssets,
) -> impl Bundle {
    let assets = tile_assets.get_assets(&tile_type);

    (
        tile_type,
        Visibility::Visible,
        Transform::from_xyz(world_pos.x, world_pos.y, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_6)),
        children![
            (Mesh2d(hex_border_mesh), MeshMaterial2d(black_material)),
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
    )
}

// ============================================================================
// Constants
// ============================================================================

pub const COLORS: [Color; 6] = [
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

// ============================================================================
// Components
// ============================================================================

#[derive(Component, PartialEq, Debug)]
pub enum TileType {
    Wall,
    MaintainSpeed,
    SlowDown,
    TurnCounterclockwise,
    TurnClockwise,
    Goal,
}

#[derive(Component)]
pub struct TileFill;

#[derive(Component)]
pub struct TileCoordinateText;

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource)]
pub struct TileAssets {
    pub hex_mesh: Handle<Mesh>,
    pub line_material: Handle<ColorMaterial>,
    pub wall: TileTypeAssets,
    pub maintain_speed: TileTypeAssets,
    pub slow_down: TileTypeAssets,
    pub turn_counterclockwise: TileTypeAssets,
    pub turn_clockwise: TileTypeAssets,
    pub goal: TileTypeAssets,
}

pub struct TileTypeAssets {
    pub material: Handle<ColorMaterial>,
    pub hover_material: Handle<ColorMaterial>,
}

impl TileAssets {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        hex_grid: &HexGrid,
    ) -> Self {
        let border_thickness = 1.0;
        TileAssets {
            hex_mesh: meshes.add(RegularPolygon::new(
                hex_grid.hex_radius - border_thickness,
                6,
            )),
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
        }
    }

    pub fn get_assets(&self, tile_type: &TileType) -> &TileTypeAssets {
        match tile_type {
            TileType::Wall => &self.wall,
            TileType::MaintainSpeed => &self.maintain_speed,
            TileType::SlowDown => &self.slow_down,
            TileType::TurnCounterclockwise => &self.turn_counterclockwise,
            TileType::TurnClockwise => &self.turn_clockwise,
            TileType::Goal => &self.goal,
        }
    }
}

// ============================================================================
// Systems
// ============================================================================

/// System to change tile type based on keyboard input when hovering over a tile
pub fn change_tile_type(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    grid: Single<&HexGrid>,
    input: Res<ButtonInput<KeyCode>>,
    mut tiles: Query<(&mut TileType, &Transform)>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) else {
        return;
    };
    if let Some(hex_coord) = world_to_hex(world_pos, *grid) {
        let Some((mut tile_type, _)) = tiles.iter_mut().find(|(_, transform)| {
            world_to_hex(transform.translation.truncate(), *grid).as_ref() == Some(&hex_coord)
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

/// On pressing the `~` key, toggle the visibility of the tile coordinates
pub fn toggle_tile_coordinates(
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

/// Updates the tile fill material for hover effects.
pub fn update_tile_hover_material(
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
    let assets = tile_assets.get_assets(tile_type);
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

// ============================================================================
// Physics
// ============================================================================

/// Computes the new velocity after applying all tile effects at the given position.
/// This is the core physics logic shared by both real-time simulation and trajectory prediction.
pub fn compute_tile_effects(
    pos: Vec2,
    velocity: &crate::Velocity,
    tiles: &[(&TileType, Vec2)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    samples: u32,
    stone_radius: f32,
) -> crate::Velocity {
    let mut rotation_angle = 0.0_f32;
    let mut total_drag = 0.0_f32;
    let mut new_velocity = velocity.clone();

    for &(tile_type, tile_world_pos) in tiles {
        let overlap_ratio = intersection::circle_hexagon_overlap_ratio(
            pos,
            stone_radius,
            tile_world_pos,
            hex_grid.hex_radius,
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
