use bevy::prelude::*;

use crate::stone::{Stone, Velocity};

const EMBER_SEED: u32 = 12345;

#[derive(Component)]
pub struct TrailDot {
    pub ttl: f32,
    pub ttl0: f32,
}

/// Simple pseudo-random number generator for trail effects
fn rand01() -> f32 {
    let seed = EMBER_SEED;
    let seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    ((seed >> 16) & 0x7fff) as f32 / 32767.0
}

/// System that spawns fire trail particles behind moving stones.
pub fn spawn_fire_trail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut stone_query: Query<(&mut Stone, &Velocity, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut stone, velocity, transform) in &mut stone_query {
        let speed = velocity.0.length();

        if speed <= 5.0 {
            continue;
        }

        // 0..1 based on speed (tweak these 2 numbers freely)
        let t = ((speed - 20.0) / 450.0).clamp(0.0, 1.0);

        // MUCH less subtle: more frequent trail
        // slow ~0.04s, fast ~0.01s
        let interval = 0.04 - 0.03 * t;

        stone.trail_accum += dt;
        if stone.trail_accum < interval {
            continue;
        }
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
        let jx = (rand01() - 0.5) * j;
        let jy = (rand01() - 0.5) * j;

        let base_x = transform.translation.x + behind.x + jx;
        let base_y = transform.translation.y + behind.y + jy;

        // --- Main flame streak (orange/red) ---
        let glow_r = stone.radius * (0.55 + 0.55 * t);
        let glow_ttl = 0.22 + 0.22 * t;
        let glow_alpha = 0.14 + 0.45 * t;

        // Fire gradient: slow = red/orange, fast = more yellow
        let glow_color = Color::srgba(1.0, 0.20 + 0.55 * t, 0.05, glow_alpha);

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
            Pickable::IGNORE,
        ));

        // --- Hot core streak (yellow/white), often ---
        if rand01() < (0.55 + 0.25 * t) {
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
                Pickable::IGNORE,
            ));
        }

        // --- Occasional ember speck (small red dot) ---
        if rand01() < (0.22 + 0.18 * t) {
            let ember_r = stone.radius * 0.10;
            let ember_ttl = 0.28 + 0.15 * t;
            let ember_alpha = 0.10 + 0.20 * t;

            let ember_color = Color::srgba(1.0, 0.10, 0.05, ember_alpha);

            let sx = (rand01() - 0.5) * (stone.radius * 1.2);
            let sy = (rand01() - 0.5) * (stone.radius * 1.2);

            commands.spawn((
                TrailDot {
                    ttl: ember_ttl,
                    ttl0: ember_ttl,
                },
                Mesh2d(meshes.add(Circle::new(ember_r))),
                MeshMaterial2d(materials.add(ember_color)),
                Transform::from_xyz(base_x + sx, base_y + sy, 2.02),
                Pickable::IGNORE,
            ));
        }
    }
}

/// System that fades and despawns trail dots over time.
pub fn update_fire_trail(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut trail_dots: Query<(Entity, &mut TrailDot, &MeshMaterial2d<ColorMaterial>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut dot, mat_handle) in &mut trail_dots {
        dot.ttl -= dt;
        if dot.ttl <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Fade out over lifetime with a nicer curve
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let t = (dot.ttl / dot.ttl0).clamp(0.0, 1.0);
            let fade = t * t; // holds brightness then drops
            mat.color.set_alpha((mat.color.alpha().min(1.0)) * fade);
        }
    }
}
