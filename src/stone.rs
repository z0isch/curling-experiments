use bevy::prelude::*;

use crate::UiState;
use crate::hex_grid::{HexCoordinate, HexGrid, hex_to_world};
use crate::intersection;
use crate::tile::{TileType, compute_tile_effects};

#[derive(Component, Clone)]
pub struct Stone;

#[derive(Component, Clone)]
pub struct Velocity(pub Vec2);

pub const STONE_RADIUS: f32 = 10.0;

/// Returns a stone bundle at the given hex coordinate with the specified velocity
pub fn stone(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    grid: &HexGrid,
    hex_coord: HexCoordinate,
    velocity: Vec2,
) -> impl Bundle {
    let black_material = materials.add(Color::BLACK);
    let stone_mesh = meshes.add(Circle::new(STONE_RADIUS));
    let stone_world_pos = hex_to_world(&hex_coord, grid);
    (
        Stone,
        Velocity(velocity),
        Mesh2d(stone_mesh),
        MeshMaterial2d(black_material),
        Transform::from_xyz(stone_world_pos.x, stone_world_pos.y, 3.0),
    )
}

pub fn update_stone_position(
    mut stone: Single<(&Velocity, &mut Transform), With<Stone>>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    time: Res<Time>,
    grid: Single<&HexGrid>,
) {
    // If velocity is zero and on the goal tile, center it in the hex
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
pub fn apply_tile_velocity_effects(
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
