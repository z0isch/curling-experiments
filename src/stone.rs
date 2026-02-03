use bevy::prelude::*;

use crate::DebugUIState;
use crate::hex_grid::{hex_to_world, HexCoordinate, HexGrid};
use crate::tile::{compute_tile_effects, TileType};

#[derive(Component, Clone, Debug)]
pub struct Stone {
    pub radius: f32,
    pub trail_accum: f32,
    pub ember_seed: u32, // tiny deterministic jitter, no RNG crate needed
}

#[derive(Component, Clone)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct TrailDot {
    pub ttl: f32,
    pub ttl0: f32,
}

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
            ember_seed: 0x1234_5678,
        },
        Velocity(velocity),
        Mesh2d(stone_mesh),
        MeshMaterial2d(black_material),
        Transform::from_xyz(stone_world_pos.x, stone_world_pos.y, 3.0),
    )
}

// Small deterministic "random" helper (no external crate)
fn next_u32(seed: &mut u32) -> u32 {
    // xorshift32
    let mut x = *seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *seed = x;
    x
}

fn rand01(seed: &mut u32) -> f32 {
    (next_u32(seed) as f32) / (u32::MAX as f32)
}

pub fn update_stone_position(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut stone: Query<(&mut Stone, &mut Velocity, &mut Transform), With<Stone>>,
    // Trail cleanup happens here too, so you don't need another system
    mut trail_dots: Query<(Entity, &mut TrailDot, &MeshMaterial2d<ColorMaterial>)>,
    tiles: Query<(&TileType, &Transform), Without<Stone>>,
    time: Res<Time>,
    debug_ui_state: Res<DebugUIState>,
) {
    let dt = time.delta_secs();

    // --- Fade & despawn trail dots (kept here so no extra systems required) ---
    for (e, mut dot, mat_handle) in &mut trail_dots {
        dot.ttl -= dt;
        if dot.ttl <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }

        // Fade out over lifetime with a nicer curve
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let t = (dot.ttl / dot.ttl0).clamp(0.0, 1.0);
            let fade = t * t; // holds brightness then drops
            mat.color.set_alpha((mat.color.alpha().min(1.0)) * fade);
        }
    }

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

        // --- Speed-based fire trail spawn ---
        if speed > 5.0 {
            // 0..1 based on speed (tweak these 2 numbers freely)
            let t = ((speed - 20.0) / 450.0).clamp(0.0, 1.0);

            // MUCH less subtle: more frequent trail
            // slow ~0.04s, fast ~0.01s
            let interval = 0.04 - 0.03 * t;

            stone.trail_accum += dt;
            if stone.trail_accum >= interval {
                stone.trail_accum = 0.0;

                let dir = velocity.0.normalize_or_zero();
                let angle = dir.y.atan2(dir.x);

                // Put the flame further behind the stone so it reads like a tail
                let behind = if dir == Vec2::ZERO {
                    Vec2::ZERO
                } else {
                    -dir * (stone.radius * (0.9 + 0.9 * t))
                };

                // Tiny jitter so it licks around like flame
                let j = stone.radius * (0.40 + 0.50 * t);
                let jx = (rand01(&mut stone.ember_seed) - 0.5) * j;
                let jy = (rand01(&mut stone.ember_seed) - 0.5) * j;

                let base_x = transform.translation.x + behind.x + jx;
                let base_y = transform.translation.y + behind.y + jy;

                // --- Main flame streak (orange/red) ---
                let glow_r = stone.radius * (0.55 + 0.55 * t);
                let glow_ttl = 0.22 + 0.22 * t;
                let glow_alpha = 0.14 + 0.45 * t;

                // Fire gradient: slow = red/orange, fast = more yellow
                let glow_color = Color::srgba(
                    1.0,
                    0.20 + 0.55 * t,
                    0.05,
                    glow_alpha,
                );

                commands.spawn((
                    TrailDot {
                        ttl: glow_ttl,
                        ttl0: glow_ttl,
                    },
                    Mesh2d(meshes.add(Circle::new(glow_r))),
                    MeshMaterial2d(materials.add(glow_color)),
                    Transform {
                        translation: Vec3::new(base_x, base_y, 2.0),
                        rotation: Quat::from_rotation_z(angle),
                        // Stretch along motion to look flamey (not circular)
                        scale: Vec3::new(2.2 + 3.2 * t, 0.28, 1.0),
                    },
                ));

                // --- Hot core streak (yellow/white), often ---
                if rand01(&mut stone.ember_seed) < (0.55 + 0.25 * t) {
                    let core_r = stone.radius * (0.22 + 0.18 * t);
                    let core_ttl = 0.12 + 0.10 * t;
                    let core_alpha = 0.18 + 0.45 * t;

                    let core_color = Color::srgba(1.0, 0.95, 0.65, core_alpha);

                    commands.spawn((
                        TrailDot {
                            ttl: core_ttl,
                            ttl0: core_ttl,
                        },
                        Mesh2d(meshes.add(Circle::new(core_r))),
                        MeshMaterial2d(materials.add(core_color)),
                        Transform {
                            translation: Vec3::new(base_x, base_y, 2.05),
                            rotation: Quat::from_rotation_z(angle),
                            scale: Vec3::new(1.6 + 2.2 * t, 0.22, 1.0),
                        },
                    ));
                }

                // --- Occasional ember speck (small red dot) ---
                if rand01(&mut stone.ember_seed) < (0.22 + 0.18 * t) {
                    let ember_r = stone.radius * 0.10;
                    let ember_ttl = 0.28 + 0.15 * t;
                    let ember_alpha = 0.10 + 0.20 * t;

                    let ember_color = Color::srgba(1.0, 0.10, 0.05, ember_alpha);

                    let sx = (rand01(&mut stone.ember_seed) - 0.5) * (stone.radius * 1.2);
                    let sy = (rand01(&mut stone.ember_seed) - 0.5) * (stone.radius * 1.2);

                    commands.spawn((
                        TrailDot {
                            ttl: ember_ttl,
                            ttl0: ember_ttl,
                        },
                        Mesh2d(meshes.add(Circle::new(ember_r))),
                        MeshMaterial2d(materials.add(ember_color)),
                        Transform::from_xyz(base_x + sx, base_y + sy, 2.02),
                    ));
                }
            }
        }

        // If close enough to the goal and moving slow enough, snap to goal center
        if let Some(goal_center) = goal_pos {
            let stone_pos = transform.translation.truncate();
            let distance_to_goal = stone_pos.distance(goal_center);

            if distance_to_goal < debug_ui_state.snap_distance && speed < debug_ui_state.snap_velocity {
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
