use bevy::prelude::*;

use crate::UiState;
use crate::hex_grid::{HexCoordinate, HexGrid, hex_to_world};
use crate::tile::{TileType, compute_tile_effects};

#[derive(Component, Clone)]
pub struct Stone {
    pub radius: f32,
}

#[derive(Component, Clone)]
pub struct Velocity(pub Vec2);

/// Returns a stone bundle at the given hex coordinate with the specified velocity
pub fn stone(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid: &HexGrid,
    hex_coord: &HexCoordinate,
    velocity: Vec2,
    radius: f32,
) -> impl Bundle {
    let black_material = materials.add(Color::BLACK);
    let stone_mesh = meshes.add(Circle::new(radius));
    let stone_world_pos = hex_to_world(hex_coord, grid);
    (
        Stone { radius },
        Velocity(velocity),
        Mesh2d(stone_mesh),
        MeshMaterial2d(black_material),
        Transform::from_xyz(stone_world_pos.x, stone_world_pos.y, 3.0),
    )
}

pub fn update_stone_position(
    mut stone: Single<(&Stone, &Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    let delta = stone.1.0 * time.delta_secs();
    stone.2.translation += delta.extend(0.);
}

/// System that modifies stone velocity based on tile types it overlaps with.
/// Uses circle_hexagon_overlap_ratio as a multiplicative factor for the effect strength.
pub fn apply_tile_velocity_effects(
    mut stone: Single<(&Stone, &mut Velocity, &mut Transform)>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    grid: Single<&HexGrid>,
    ui_state: Res<UiState>,
) {
    let tile_data: Vec<_> = tiles
        .iter()
        .map(|(tile_type, transform)| (tile_type, transform.translation.truncate()))
        .collect();
    *stone.1 = compute_tile_effects(
        stone.2.translation.truncate(),
        &stone.1,
        &tile_data,
        *grid,
        ui_state.drag_coefficient,
        stone.0.radius,
    );
}
