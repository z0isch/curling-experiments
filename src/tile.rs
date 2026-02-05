use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

use crate::debug_ui::DebugUIState;
use crate::hex_grid::HexGrid;
use crate::intersection;
use crate::level::Facing;

// ============================================================================
// Custom Scratch-Off Material
// ============================================================================

/// Custom material that creates a scratch-off reveal effect
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ScratchOffMaterial {
    #[uniform(0)]
    pub top_color: LinearRgba,
    #[uniform(0)]
    pub reveal_color: LinearRgba,
    #[uniform(0)]
    pub progress: f32,
}

impl Material2d for ScratchOffMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/scratch_off.wgsl".into()
    }
}

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
    scratch_materials: &mut Assets<ScratchOffMaterial>,
) -> impl Bundle {
    // Create a unique scratch-off material for this tile
    let top_color = get_tile_color(&tile_type).to_linear();
    let reveal_color = COLORS[0].to_linear(); // MaintainSpeed color

    let scratch_material = scratch_materials.add(ScratchOffMaterial {
        top_color,
        reveal_color,
        progress: 0.0,
    });

    let (arrow_visibility, arrow_rotation) = match &tile_type {
        TileType::SpeedUp(facing) => (
            Visibility::Visible,
            Quat::from_rotation_z(facing.to_angle() - std::f32::consts::FRAC_PI_6),
        ),
        _ => (Visibility::Hidden, Quat::IDENTITY),
    };

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
                MeshMaterial2d(scratch_material),
                Transform::from_xyz(0., 0., 1.0),
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                }
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
                Mesh2d(tile_assets.arrow_mesh.clone()),
                MeshMaterial2d(tile_assets.arrow_material.clone()),
                Transform::from_xyz(0., 0., 3.0).with_rotation(arrow_rotation),
                arrow_visibility,
            ),
        ],
    )
}

// ============================================================================
// Constants
// ============================================================================

pub const COLORS: [Color; 6] = [
    Color::srgb(238.0 / 255.0, 249.0 / 255.0, 1.), // rgb(238, 249, 255)
    Color::srgb(35.0 / 255.0, 221. / 255., 1.),    // rgb(35, 221, 255)
    Color::srgb(78.0 / 255.0, 238.0 / 255.0, 179.0 / 255.0), //rgb(78, 238, 179)
    Color::srgb(12.0 / 255.0, 60.0 / 255.0, 251.0 / 255.0), //rgb(12, 60, 251)
    Color::srgb(221.0 / 255.0, 104.0 / 255.0, 210.0 / 255.0), //rgb(221, 104, 210)
    Color::srgb(1., 60.0 / 255.0, 90.0 / 255.0),   // rgb(255,53,79)
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
    SpeedUp(Facing),
}

#[derive(Component)]
pub struct TileFill;

#[derive(Component)]
pub struct TileCoordinateText;

#[derive(Component, Debug)]
pub struct TileDragging {
    pub last_position: Option<Vec2>,
    pub distance_dragged: f32,
    pub tile_type: TileType,
}

#[derive(Component)]
pub struct MouseHover;

// ============================================================================
// Resources
// ============================================================================

/// Tracks the current tile type being used for dragging/painting
#[derive(Resource)]
pub struct CurrentDragTileType(pub TileType);

#[derive(Resource)]
pub struct TileAssets {
    pub hex_mesh: Handle<Mesh>,
    pub hex_border_mesh: Handle<Mesh>,
    pub arrow_mesh: Handle<Mesh>,
    pub border_material: Handle<ColorMaterial>,
    pub line_material: Handle<ColorMaterial>,
    pub arrow_material: Handle<ColorMaterial>,
}

impl TileAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        hex_grid: &HexGrid,
    ) -> Self {
        let border_thickness = 1.0;

        let mut arrow_mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        arrow_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[20.0, 0.0, 0.0], [-7.5, 12.5, 0.0], [-7.5, -12.5, 0.0]],
        );
        arrow_mesh.insert_indices(Indices::U32(vec![0, 1, 2]));

        TileAssets {
            hex_mesh: meshes.add(RegularPolygon::new(
                hex_grid.hex_radius - border_thickness,
                6,
            )),
            hex_border_mesh: meshes.add(RegularPolygon::new(hex_grid.hex_radius, 6)),
            arrow_mesh: meshes.add(arrow_mesh),
            border_material: materials.add(Color::BLACK),
            line_material: materials.add(COLORS[5]),
            arrow_material: materials.add(COLORS[5]),
        }
    }
}

// ============================================================================
// Systems
// ============================================================================

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

/// Returns the base color for a given tile type
fn get_tile_color(tile_type: &TileType) -> Color {
    match tile_type {
        TileType::Wall => COLORS[3],
        TileType::MaintainSpeed => COLORS[0],
        TileType::SlowDown => COLORS[1],
        TileType::TurnCounterclockwise => COLORS[2],
        TileType::TurnClockwise => COLORS[4],
        TileType::Goal => COLORS[5],
        TileType::SpeedUp(_facing) => COLORS[0],
    }
}

pub fn update_tile_material(
    tile_query: Query<(Entity, &TileType, Option<&TileDragging>)>,
    children_query: Query<&Children>,
    _tile_assets: Res<TileAssets>,
    debug_ui_state: Res<DebugUIState>,
    current_drag_tile_type: Res<CurrentDragTileType>,
    mut scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
    fill_query: Query<&MeshMaterial2d<ScratchOffMaterial>, With<TileFill>>,
) {
    for (entity, tile_type, tile_dragging) in tile_query {
        match tile_type {
            TileType::Wall => continue,
            TileType::Goal => continue,
            TileType::SpeedUp(_facing) => continue,
            _ => {}
        }
        let Ok(children) = children_query.get(entity) else {
            continue;
        };

        // Calculate linear progress for scratch-off effect
        let linear_progress = if let Some(dragging) = tile_dragging {
            if debug_ui_state.min_sweep_distance > 0.0 {
                (dragging.distance_dragged / debug_ui_state.min_sweep_distance).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Apply ease-out curve: ramp up quickly at the start, slow down toward the end
        // sqrt makes the first few percent much more impactful (e.g., 1% -> 10%, 4% -> 20%)
        let eased_progress = linear_progress.sqrt();

        // It's hard to see when the tile is almost complete, so we scale it down to 50%
        let display_progress = if linear_progress >= 1.0 {
            1.0
        } else {
            eased_progress * 0.50
        };

        // Get the reveal color from either the tile's dragging state or the current drag tile type
        let reveal_tile_type = tile_dragging
            .map(|d| &d.tile_type)
            .unwrap_or(&current_drag_tile_type.0);
        let reveal_color = get_tile_color(reveal_tile_type).to_linear();

        for child in children.iter() {
            // Update scratch-off material properties
            if let Ok(mesh_material) = fill_query.get(child)
                && let Some(material) = scratch_materials.get_mut(&mesh_material.0)
            {
                material.progress = display_progress;
                material.reveal_color = reveal_color;
            }
        }
    }
}

pub fn update_tile_type(
    debug_ui_state: Res<DebugUIState>,
    tiles: Query<(&TileDragging, &mut TileType)>,
) {
    for (tile_dragging, mut tile_type) in tiles {
        if tile_dragging.distance_dragged > debug_ui_state.min_sweep_distance {
            *tile_type = tile_dragging.tile_type.clone();
        }
    }
}

//=============================================================================
// Observers
//=============================================================================

pub fn on_pointer_over(over: On<Pointer<Over>>, mut commands: Commands) {
    commands.entity(over.entity).insert(MouseHover);
}

pub fn on_pointer_out(out: On<Pointer<Out>>, mut commands: Commands) {
    commands.entity(out.entity).remove::<MouseHover>();
}

pub fn on_tile_drag_enter(
    drag_enter: On<Pointer<DragEnter>>,
    mut commands: Commands,
    mut tile_dragging_q: Query<Option<&mut TileDragging>>,
    current_drag_tile_type: Res<CurrentDragTileType>,
) {
    if let Ok(Some(mut tile_dragging)) = tile_dragging_q.get_mut(drag_enter.entity) {
        tile_dragging.last_position = Some(drag_enter.pointer_location.position);
    } else {
        commands.entity(drag_enter.entity).insert(TileDragging {
            last_position: Some(drag_enter.pointer_location.position),
            distance_dragged: 0.0,
            tile_type: current_drag_tile_type.0.clone(),
        });
    }
}

pub fn on_tile_dragging(
    drag: On<Pointer<Drag>>,
    mut tile: Single<(&mut TileDragging, &TileType), With<MouseHover>>,
) {
    match tile.1 {
        TileType::Wall => return,
        TileType::Goal => return,
        TileType::SpeedUp(_facing) => return,
        _ => {}
    }
    if let Some(last_position) = tile.0.last_position {
        tile.0.distance_dragged += (drag.pointer_location.position - last_position).length();
        tile.0.last_position = Some(drag.pointer_location.position);
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
///
/// When a tile is being dragged, the effects are blended between the current tile type
/// and the target tile type based on the drag progress.
pub fn compute_tile_effects(
    stone_pos: Vec2,
    velocity: &crate::stone::Velocity,
    tiles: &[(&TileType, Vec2, Option<&TileDragging>)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    stone_radius: f32,
    slow_down_factor: f32,
    rotation_factor: f32,
    min_sweep_distance: f32,
    speed_up_factor: f32,
) -> crate::stone::Velocity {
    let mut new_velocity = velocity.0;

    let mut rotation_angle: f32 = 0.0;
    let mut total_drag: f32 = 0.0;

    for (tile_type, tile_position, maybe_dragging) in tiles {
        let ratio = intersection::ratio_circle_area_inside_hexagon(
            stone_pos,
            stone_radius,
            *tile_position,
            hex_grid.hex_radius - 2.,
            60,
        );
        if ratio < 0.01 {
            continue;
        }

        // Calculate weights for blending between current type and drag target type
        let (current_weight, drag_weight, drag_tile_type) = match *maybe_dragging {
            Some(dragging) => {
                if dragging.tile_type == **tile_type {
                    (1.0, 0.0, None)
                } else {
                    let progress = if min_sweep_distance > 0.0 {
                        (dragging.distance_dragged / min_sweep_distance).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    (1.0 - progress, progress, Some(&dragging.tile_type))
                }
            }
            None => (1.0, 0.0, None),
        };

        // Helper closure to apply effects for a single tile type with a given weight
        let mut apply_tile_effect = |tile_type: &TileType, weight: f32| {
            if weight < 0.001 {
                return;
            }
            let weighted_ratio = ratio * weight;

            match tile_type {
                TileType::Wall => {
                    // Use proper hexagon edge normal instead of radial direction
                    let wall_normal = hex_edge_normal(stone_pos - tile_position);
                    let dot = new_velocity.dot(wall_normal);
                    // Only reflect if moving toward the wall
                    if dot < 0.0 {
                        // Store original speed to preserve magnitude after reflection
                        let original_speed = new_velocity.length();
                        // Apply partial reflection based on weight
                        new_velocity -= 2.0 * dot * wall_normal * weight;
                        // Re-normalize to original speed to prevent floating-point drift
                        let new_speed = new_velocity.length();
                        if new_speed > 1e-10 {
                            new_velocity *= original_speed / new_speed;
                        }
                    }
                }
                TileType::MaintainSpeed => {
                    total_drag += drag_coefficient * weighted_ratio;
                }
                TileType::SlowDown => {
                    total_drag += drag_coefficient * weighted_ratio * slow_down_factor;
                }
                TileType::TurnCounterclockwise => {
                    rotation_angle += rotation_factor * weighted_ratio;
                    total_drag += drag_coefficient * weighted_ratio;
                }
                TileType::TurnClockwise => {
                    rotation_angle -= rotation_factor * weighted_ratio;
                    total_drag += drag_coefficient * weighted_ratio;
                }
                TileType::Goal => {
                    // Pull towards the center of the goal
                    let to_center = tile_position - stone_pos;
                    let distance = to_center.length();
                    if distance > 1e-10 {
                        let direction = to_center / distance;
                        // Pull strength proportional to how much of stone is inside
                        let pull_strength = 0.5 * weighted_ratio;
                        new_velocity += direction * pull_strength;
                    }
                    total_drag += drag_coefficient * slow_down_factor * weighted_ratio;
                }
                TileType::SpeedUp(facing) => {
                    // Pull towards the center of the goal
                    let to_center = tile_position - stone_pos;
                    let distance = to_center.length();
                    if distance > (hex_grid.hex_radius * 1. / 4.) {
                        let direction = to_center / distance;
                        // Pull strength proportional to how much of stone is inside
                        let pull_strength = 0.5 * weighted_ratio;
                        new_velocity += direction * pull_strength;
                    } else {
                        let direction = facing.to_vector();
                        new_velocity = direction * speed_up_factor;
                    }
                }
            }
        };

        // Apply effects from the current tile type
        apply_tile_effect(tile_type, current_weight);

        // Apply effects from the drag target type (if dragging)
        if let Some(drag_type) = drag_tile_type {
            apply_tile_effect(drag_type, drag_weight);
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
