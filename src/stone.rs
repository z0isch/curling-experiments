use bevy::prelude::*;

use crate::DebugUIState;
use crate::hex_grid::{HexCoordinate, HexGrid, hex_to_world};
use crate::tile::{TileType, compute_tile_effects};

#[derive(Component, Clone, Debug)]
pub struct Stone {
    pub radius: f32,
    pub trail_accum: f32,
    pub ember_seed: u32,
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
        Stone {
            radius,
            trail_accum: 0.0,
            ember_seed: 12345,
        },
        Velocity(velocity),
        Mesh2d(stone_mesh),
        MeshMaterial2d(black_material),
        Transform::from_xyz(stone_world_pos.x, stone_world_pos.y, 3.0),
    )
}

pub fn update_stone_position(
    mut stone: Query<(&mut Stone, &mut Velocity, &mut Transform), With<Stone>>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    time: Res<Time<Fixed>>,
    debug_ui_state: Res<DebugUIState>,
) {
    let dt = time.delta_secs();

    // Find goal tile position
    let goal_pos = tiles.iter().find_map(|(tile_type, transform)| {
        if *tile_type == TileType::Goal {
            Some(transform.translation.truncate())
        } else {
            None
        }
    });

    for (mut stone, mut velocity, mut transform) in &mut stone {
        // Move stone
        transform.translation += (velocity.0 * dt).extend(0.0);

        let speed = velocity.0.length();

        // If close enough to the goal and moving slow enough, snap to goal center
        if let Some(goal_center) = goal_pos {
            let stone_pos = transform.translation.truncate();
            let distance_to_goal = stone_pos.distance(goal_center);

            if distance_to_goal < debug_ui_state.snap_distance
                && speed < debug_ui_state.snap_velocity
            {
                transform.translation.x = goal_center.x;
                transform.translation.y = goal_center.y;
                velocity.0 = Vec2::ZERO;
                stone.trail_accum = 0.0;
            }
        }
    }
}

/// Checks if two stones collide and returns their new velocities if they do.
pub fn resolve_collision(
    pos1: Vec2,
    vel1: &Velocity,
    radius1: f32,
    pos2: Vec2,
    vel2: &Velocity,
    radius2: f32,
) -> Option<(Velocity, Velocity)> {
    let distance_squared = pos1.distance_squared(pos2);
    let min_distance = radius1 + radius2;

    // Check if circles are overlapping
    if distance_squared >= min_distance * min_distance {
        return None;
    }

    // Calculate collision normal (from stone1 to stone2)
    let collision_normal = (pos2 - pos1).normalize_or_zero();

    // If stones are at the exact same position, use a default direction
    let collision_normal = if collision_normal == Vec2::ZERO {
        Vec2::X
    } else {
        collision_normal
    };

    // Calculate relative velocity
    let relative_velocity = vel1.0 - vel2.0;

    // Calculate relative velocity along the collision normal
    let velocity_along_normal = relative_velocity.dot(collision_normal);

    // Only resolve if stones are approaching each other
    if velocity_along_normal <= 0.0 {
        return None;
    }

    // Coefficient of restitution (1.0 = perfectly elastic, 0.0 = perfectly inelastic)
    let restitution = 0.85;

    // For equal masses: impulse = (1 + e) * v_rel_normal / 2
    let impulse_scalar = (1.0 + restitution) * velocity_along_normal / 2.0;
    let impulse = impulse_scalar * collision_normal;

    let new_vel1 = Velocity(vel1.0 - impulse);
    let new_vel2 = Velocity(vel2.0 + impulse);

    Some((new_vel1, new_vel2))
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
