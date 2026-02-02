use bevy::prelude::*;

use crate::DebugUIState;
use crate::hex_grid::{HexCoordinate, HexGrid, hex_to_world};
use crate::tile::{TileType, compute_tile_effects};

#[derive(Component, Clone, Debug)]
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
    stone: Query<(&Velocity, &mut Transform), With<Stone>>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in stone {
        let delta = velocity.0 * time.delta_secs();
        transform.translation += delta.extend(0.);
    }
}

/// Checks if two stones collide and returns their new velocities if they do.
/// Returns None if no collision occurs.
pub fn resolve_collision(
    pos1: Vec2,
    vel1: &Velocity,
    radius1: f32,
    pos2: Vec2,
    vel2: &Velocity,
    radius2: f32,
) -> Option<(Velocity, Velocity)> {
    let should_collide = (radius1 + radius2).powi(2) > pos1.distance_squared(pos2);

    if should_collide {
        if vel2.0.length_squared() > 0.0 {
            Some((vel2.clone(), Velocity(Vec2::ZERO)))
        } else {
            Some((Velocity(Vec2::ZERO), vel1.clone()))
        }
    } else {
        None
    }
}

pub fn apply_stone_collision(mut stone_query: Query<(&Stone, &mut Velocity, &Transform)>) {
    let mut combinations = stone_query.iter_combinations_mut();
    while let Some(
        [
            (stone1, mut velocity1, transform1),
            (stone2, mut velocity2, transform2),
        ],
    ) = combinations.fetch_next()
    {
        if let Some((new_vel1, new_vel2)) = resolve_collision(
            transform1.translation.truncate(),
            &velocity1,
            stone1.radius,
            transform2.translation.truncate(),
            &velocity2,
            stone2.radius,
        ) {
            *velocity1 = new_vel1;
            *velocity2 = new_vel2;
        }
    }
}

/// System that modifies stone velocity based on tile types it overlaps with.
pub fn apply_tile_velocity_effects(
    stone_query: Query<(&Stone, &mut Velocity, &Transform)>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    grid: Single<&HexGrid>,
    debug_ui_state: Res<DebugUIState>,
) {
    for (stone, mut velocity, transform) in stone_query {
        let tile_data: Vec<_> = tiles
            .iter()
            .map(|(tile_type, transform)| (tile_type, transform.translation.truncate()))
            .collect();
        *velocity = compute_tile_effects(
            transform.translation.truncate(),
            &velocity,
            &tile_data,
            *grid,
            debug_ui_state.drag_coefficient,
            stone.radius,
            debug_ui_state.slow_down_factor,
            debug_ui_state.rotation_factor,
        );
    }
}
