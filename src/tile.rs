use bevy::prelude::*;

use crate::hex_grid::{HexGrid, world_to_hex};
use crate::{DebugUIState, intersection};

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
    tile_assets: &TileAssets,
) -> impl Bundle {
    let assets = tile_assets.get_assets(&tile_type);

    (
        tile_type,
        Visibility::Visible,
        Transform::from_xyz(world_pos.x, world_pos.y, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_6)),
        children![
            (
                Mesh2d(tile_assets.hex_border_mesh.clone()),
                MeshMaterial2d(tile_assets.border_material.clone()),
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

#[derive(Component, PartialEq, Debug, Clone)]
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

#[derive(Component, Debug)]
pub struct TileDragging {
    pub last_position: Vec2,
    pub distance_dragged: f32,
}

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource)]
pub struct TileAssets {
    pub hex_mesh: Handle<Mesh>,
    pub hex_border_mesh: Handle<Mesh>,
    pub border_material: Handle<ColorMaterial>,
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
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        hex_grid: &HexGrid,
    ) -> Self {
        let border_thickness = 1.0;
        TileAssets {
            hex_mesh: meshes.add(RegularPolygon::new(
                hex_grid.hex_radius - border_thickness,
                6,
            )),
            hex_border_mesh: meshes.add(RegularPolygon::new(hex_grid.hex_radius, 6)),
            border_material: materials.add(Color::BLACK),
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

pub fn update_tile_type(
    debug_ui_state: Res<DebugUIState>,
    tiles: Query<(Entity, &TileDragging, &mut TileType)>,
    children_query: Query<&Children>,
    mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>,
    tile_assets: Res<TileAssets>,
) {
    for (entity, tile_dragging, mut tile_type) in tiles {
        if tile_dragging.distance_dragged > debug_ui_state.min_sweep_distance {
            *tile_type = TileType::MaintainSpeed;

            let Ok(children) = children_query.get(entity) else {
                return;
            };
            for child in children.iter() {
                let assets = tile_assets.get_assets(&tile_type);
                if let Ok(mut mesh_material) = fill_query.get_mut(child) {
                    mesh_material.0 = assets.material.clone();
                }
            }
        }
    }
}

//=============================================================================
// Observers
//=============================================================================

pub fn on_pointer_over(
    over: On<Pointer<Over>>,
    tile_type: Query<&TileType>,
    children: Query<&Children>,
    tile_assets: Res<TileAssets>,
    mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>,
) {
    update_tile_hover_material(
        over.entity,
        true,
        &tile_type,
        &children,
        &tile_assets,
        &mut fill_query,
    );
}

pub fn on_pointer_out(
    out: On<Pointer<Out>>,
    tile_type: Query<&TileType>,
    children: Query<&Children>,
    tile_assets: Res<TileAssets>,
    mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>,
) {
    update_tile_hover_material(
        out.entity,
        false,
        &tile_type,
        &children,
        &tile_assets,
        &mut fill_query,
    );
}

pub fn on_tile_drag_enter(
    drag_enter: On<Pointer<DragEnter>>,
    mut commands: Commands,
    mut tile_dragging_q: Query<Option<&mut TileDragging>>,
) {
    if let Ok(Some(mut tile_dragging)) = tile_dragging_q.get_mut(drag_enter.entity) {
        tile_dragging.last_position = drag_enter.pointer_location.position;
    } else {
        commands.entity(drag_enter.entity).insert(TileDragging {
            last_position: drag_enter.pointer_location.position,
            distance_dragged: 0.0,
        });
    }
}

pub fn on_tile_dragging(
    drag: On<Pointer<Drag>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    grid: Single<&HexGrid>,
    mut tiles: Query<(&mut TileDragging, &Transform, &TileType)>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) else {
        return;
    };
    if let Some(hex_coord) = world_to_hex(world_pos, *grid) {
        let Some((mut tile_dragging, _, tile_type)) = tiles.iter_mut().find(|(_, transform, _)| {
            world_to_hex(transform.translation.truncate(), *grid).as_ref() == Some(&hex_coord)
        }) else {
            return;
        };
        if *tile_type == TileType::Goal || *tile_type == TileType::Wall {
            return;
        }
        tile_dragging.distance_dragged +=
            (drag.pointer_location.position - tile_dragging.last_position).length();
        tile_dragging.last_position = drag.pointer_location.position;
    }
}
// ============================================================================
// Physics
// ============================================================================

/// Pre-computed edge normals for a pointy-top hexagon (vertices at 0°, 60°, 120°, etc.)
/// Each normal points outward from the center, perpendicular to its edge.
/// Edge normals are at angles: 30°, 90°, 150°, 210°, 270°, 330°
const HEX_EDGE_NORMALS: [Vec2; 6] = [
    Vec2::new(0.8660254, 0.5),   // 30° - edge between 0° and 60° vertices
    Vec2::new(0.0, 1.0),         // 90° - edge between 60° and 120° vertices
    Vec2::new(-0.8660254, 0.5),  // 150° - edge between 120° and 180° vertices
    Vec2::new(-0.8660254, -0.5), // 210° - edge between 180° and 240° vertices
    Vec2::new(0.0, -1.0),        // 270° - edge between 240° and 300° vertices
    Vec2::new(0.8660254, -0.5),  // 330° - edge between 300° and 360° vertices
];

/// Returns the outward normal of the hexagon edge closest to the given relative position.
/// `relative_pos` is the position relative to the hexagon center (stone_pos - hex_center).
fn hex_edge_normal(relative_pos: Vec2) -> Vec2 {
    // Get the angle of the relative position (0 to 2π)
    let angle = relative_pos.y.atan2(relative_pos.x);
    // Convert to positive angle in range [0, 2π)
    let angle = if angle < 0.0 {
        angle + std::f32::consts::TAU
    } else {
        angle
    };

    // Determine which of the 6 sectors (each 60° = π/3) the position is in
    // Sectors are: [0°,60°), [60°,120°), [120°,180°), [180°,240°), [240°,300°), [300°,360°)
    let sector = ((angle / std::f32::consts::FRAC_PI_3) as usize).min(5);

    HEX_EDGE_NORMALS[sector]
}

/// Computes the new velocity after applying all tile effects at the given position.
/// This is the core physics logic shared by both real-time simulation and trajectory prediction.
pub fn compute_tile_effects(
    stone_pos: Vec2,
    velocity: &crate::stone::Velocity,
    tiles: &[(&TileType, Vec2)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    stone_radius: f32,
    slow_down_factor: f32,
    rotation_factor: f32,
) -> crate::stone::Velocity {
    let mut new_velocity = velocity.0;

    let mut rotation_angle: f32 = 0.0;
    let mut total_drag: f32 = 0.0;

    for &(tile_type, tile_world_pos) in tiles {
        let ratio = intersection::ratio_circle_area_inside_hexagon(
            stone_pos,
            stone_radius,
            tile_world_pos,
            hex_grid.hex_radius - 2.,
            60,
        );
        if ratio < 0.01 {
            continue;
        }

        match tile_type {
            TileType::Wall => {
                // Use proper hexagon edge normal instead of radial direction
                let wall_normal = hex_edge_normal(stone_pos - tile_world_pos);
                let dot = new_velocity.dot(wall_normal);
                // Only reflect if moving toward the wall
                if dot < 0.0 {
                    // Store original speed to preserve magnitude after reflection
                    let original_speed = new_velocity.length();
                    new_velocity -= 2.0 * dot * wall_normal;
                    // Re-normalize to original speed to prevent floating-point drift
                    let new_speed = new_velocity.length();
                    if new_speed > 1e-10 {
                        new_velocity *= original_speed / new_speed;
                    }
                }
            }
            TileType::MaintainSpeed => {
                total_drag += drag_coefficient * ratio;
            }
            TileType::SlowDown => {
                total_drag += drag_coefficient * ratio * slow_down_factor;
            }
            TileType::TurnCounterclockwise => {
                rotation_angle += rotation_factor * ratio; // ~1 degree per frame
                total_drag += drag_coefficient * ratio;
            }
            TileType::TurnClockwise => {
                rotation_angle -= rotation_factor * ratio; // clockwise = negative
                total_drag += drag_coefficient * ratio;
            }
            TileType::Goal => {
                total_drag += drag_coefficient * ratio;
            }
        }
    }

    // Apply accumulated rotation to velocity vector
    if rotation_angle.abs() > 1e-10 {
        let (sin_angle, cos_angle) = rotation_angle.sin_cos();
        new_velocity = Vec2::new(
            new_velocity.x * cos_angle - new_velocity.y * sin_angle,
            new_velocity.x * sin_angle + new_velocity.y * cos_angle,
        );
    }

    // Apply accumulated drag - reduces velocity magnitude while preserving direction
    if total_drag > 0.0 {
        // Clamp drag factor to prevent velocity reversal
        let drag_factor = (1.0 - total_drag).max(0.0);
        new_velocity *= drag_factor;
    }

    crate::stone::Velocity(new_velocity)
}
