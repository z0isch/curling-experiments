use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

#[derive(Component)]
pub struct MouseTrailLine;

/// Local state for the mouse trail line (keeps last position, points, and spawned entity)
#[derive(Default)]
pub(crate) struct MouseTrailLocal {
    last_pos: Option<Vec2>,
    points: Vec<Vec2>,
    line_entity: Option<Entity>,
    released_timer: Option<f32>,
}

const TRAIL_TTL: f32 = 1.5;

/// Builds/updates a single polyline while the left mouse button is held.
/// The line persists after release and fades out over `TRAIL_TTL` seconds.
pub fn spawn_mouse_trail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut local: Local<MouseTrailLocal>,
) {
    // Settings
    let spawn_spacing = 6.0; // world units between samples
    let gold = Color::srgba(1.0, 0.84, 0.0, 1.0);
    let smoothing_subdivs = 4; // Catmull-Rom subdivisions per span
    let thickness = 10.0; // desired polyline thickness

    // Resolve cursor to world
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let cursor_pos = match window.cursor_position() {
        Some(p) => p,
        None => {
            // cursor outside window
            return;
        }
    };
    let (camera, camera_transform) = match camera_q.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let world_pos = match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        Ok(p) => p,
        Err(_) => return,
    };

    // If left mouse pressed, sample points and reset release timer
    if mouse_buttons.pressed(MouseButton::Left) {
        local.released_timer = None;

        if local.last_pos.is_none() {
            local.last_pos = Some(world_pos);
            local.points.push(world_pos);
        } else {
            let mut last = local.last_pos.unwrap();
            let mut remaining = (world_pos - last).length();
            if remaining < 0.01 {
                // negligible movement
                local.last_pos = Some(world_pos);
            } else {
                let dir = (world_pos - last).normalize_or_zero();
                while remaining > spawn_spacing {
                    last += dir * spawn_spacing;
                    remaining = (world_pos - last).length();
                    local.points.push(last);
                }
                local.points.push(world_pos);
                local.last_pos = Some(world_pos);
            }
        }

        // Build a smoothed point list using Catmull-Rom
        let smoothed = if local.points.len() >= 2 {
            catmull_rom_spline(&local.points, smoothing_subdivs)
        } else {
            local.points.clone()
        };

        // Build a smoothed point list and spawn/update a Polyline2d mesh
        if smoothed.len() >= 2 {
            // create base polyline mesh (thin)
            let mesh_handle = meshes.add(Polyline2d::new(smoothed.clone()));
            let mat_handle = materials.add(gold);

            // spawn parent entity with polyline
            if local.line_entity.is_none() {
                let ent = commands
                    .spawn((
                        MouseTrailLine,
                        Mesh2d(mesh_handle),
                        MeshMaterial2d(mat_handle.clone()),
                        Transform::from_xyz(0.0, 0.0, 6.0),
                    ))
                    .id();
                // spawn circle children along the smoothed points to thicken the appearance
                let radius = thickness * 0.5;
                let mut child_ids: Vec<Entity> = Vec::new();
                for p in &smoothed {
                    let child = commands
                        .spawn((
                            Mesh2d(meshes.add(Circle::new(radius))),
                            MeshMaterial2d(mat_handle.clone()),
                            Transform::from_xyz(p.x, p.y, 5.9),
                        ))
                        .id();
                    child_ids.push(child);
                }
                commands.entity(ent).add_children(&child_ids);
                local.line_entity = Some(ent);
            } else if let Some(ent) = local.line_entity {
                // update polyline mesh
                commands.entity(ent).insert(Mesh2d(mesh_handle));
            }
        }
    } else {
        // Mouse not pressed: start or continue release timer
        if local.line_entity.is_some() && local.released_timer.is_none() {
            local.released_timer = Some(0.0);
        }
        local.last_pos = None;
    }
}

/// Updates fade and despawn for the single trail line created above.
pub fn update_mouse_trail(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut local: Local<MouseTrailLocal>,
    time: Res<Time>,
    query: Query<(&MouseTrailLine, &MeshMaterial2d<ColorMaterial>)>,
) {
    let dt = time.delta_secs();

    if let Some(ent) = local.line_entity {
        // If entity has been despawned elsewhere, clear local
        if query.get(ent).is_err() {
            local.line_entity = None;
            local.points.clear();
            local.released_timer = None;
            return;
        }

        // Update release timer and material alpha
        if let Some(t) = &mut local.released_timer {
            *t += dt;
            let alpha = (1.0 - (*t / TRAIL_TTL)).clamp(0.0, 1.0);

            if let Ok((_, mat_handle)) = query.get(ent) {
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.color.set_alpha(alpha);
                }
            }

            if *t >= TRAIL_TTL {
                // remove line
                commands.entity(ent).despawn();
                local.line_entity = None;
                local.points.clear();
                local.released_timer = None;
            }
        } else {
            // ensure full alpha while drawing
            if let Ok((_, mat_handle)) = query.get(ent) {
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.color.set_alpha(1.0);
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Helpers: Catmull-Rom smoothing and thick polyline mesh builder
// -----------------------------------------------------------------------------

fn catmull_rom_spline(points: &Vec<Vec2>, subdivisions: usize) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.clone();
    }
    let mut out = Vec::new();
    let n = points.len();
    for i in 0..n - 1 {
        // p0 p1 p2 p3
        let p1 = points[i];
        let p2 = points[i + 1];
        let p0 = if i == 0 { p1 } else { points[i - 1] };
        let p3 = if i + 2 >= n { p2 } else { points[i + 2] };

        for s in 0..subdivisions {
            let t = s as f32 / subdivisions as f32;
            out.push(catmull_rom(p0, p1, p2, p3, t));
        }
    }
    // push last point
    out.push(*points.last().unwrap());
    out
}

fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

// (replaced by stretched-circle segments to avoid low-level mesh construction)
