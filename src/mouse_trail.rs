use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

#[derive(Component)]
pub struct MouseTrailLine;

/// Local state for the mouse trail line (keeps last position, points, and spawned entity)
#[derive(Default)]
pub(crate) struct MouseTrailLocal {
    last_pos: Option<Vec2>,
    // points for the currently-drawing segment
    active_points: Vec<Vec2>,
    // multiple trail segments (previous ones persist and fade)
    segments: Vec<TrailSegment>,
    // index of the currently-active segment in `segments`
    current_segment: Option<usize>,
}

#[derive(Clone)]
struct TrailSegment {
    parent: Entity,
    child_entities: Vec<Entity>,
    // remaining time (seconds) until fully faded and removed
    released_timer: Option<f32>,
    mat_handle: Handle<ColorMaterial>,
}

const TRAIL_TTL: f32 = 0.5;

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
        None => return,
    };
    let (camera, camera_transform) = match camera_q.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let world_pos = match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        Ok(p) => p,
        Err(_) => return,
    };

    // If left mouse pressed, sample points and either start a new segment or update the active one
    if mouse_buttons.pressed(MouseButton::Left) {
        // If there is no active segment, start one
        if local.current_segment.is_none() {
            local.active_points.clear();
            local.active_points.push(world_pos);
            local.last_pos = Some(world_pos);

            // Create a material for this segment
            let mat_handle = materials.add(gold);

            // Create a parent entity (polyline will be inserted as we get more points)
            let parent = commands
                .spawn((
                    MouseTrailLine,
                    // start without a mesh; we'll insert Mesh2d once we have >=2 points
                    MeshMaterial2d(mat_handle.clone()),
                    Transform::from_xyz(0.0, 0.0, 6.0),
                    Pickable::IGNORE,
                ))
                .id();

            // create an initial child circle at the first point
            let radius = thickness * 0.5;
            let child = commands
                .spawn((
                    Mesh2d(meshes.add(Circle::new(radius))),
                    MeshMaterial2d(mat_handle.clone()),
                    Transform::from_xyz(world_pos.x, world_pos.y, 5.9),
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(parent).add_children(&[child]);

            local.segments.push(TrailSegment {
                parent,
                child_entities: vec![child],
                released_timer: None,
                mat_handle: mat_handle.clone(),
            });
            local.current_segment = Some(local.segments.len() - 1);
        } else {
            // continue the current segment
            let idx = local.current_segment.unwrap();
            // sampling like before
            if local.last_pos.is_none() {
                local.last_pos = Some(world_pos);
                local.active_points.push(world_pos);
            } else {
                let mut last = local.last_pos.unwrap();
                let mut remaining = (world_pos - last).length();
                if remaining < 0.01 {
                    local.last_pos = Some(world_pos);
                } else {
                    let dir = (world_pos - last).normalize_or_zero();
                    while remaining > spawn_spacing {
                        last += dir * spawn_spacing;
                        remaining = (world_pos - last).length();
                        local.active_points.push(last);
                    }
                    local.active_points.push(world_pos);
                    local.last_pos = Some(world_pos);
                }
            }

            // Build smoothed points
            let smoothed = if local.active_points.len() >= 2 {
                catmull_rom_spline(&local.active_points, smoothing_subdivs)
            } else {
                local.active_points.clone()
            };

            // Update mesh if we have >=2 points
            let parent = local.segments[idx].parent;
            let seg_mat = local.segments[idx].mat_handle.clone();

            if smoothed.len() >= 2 {
                let mesh_handle = meshes.add(Polyline2d::new(smoothed.clone()));
                commands.entity(parent).insert(Mesh2d(mesh_handle));

                // ensure child entities exist and update their transforms
                let radius = thickness * 0.5;
                // spawn extra children if needed
                if local.segments[idx].child_entities.len() < smoothed.len() {
                    let mut new_children = Vec::new();
                    for p in &smoothed[local.segments[idx].child_entities.len()..] {
                        let child = commands
                            .spawn((
                                Mesh2d(meshes.add(Circle::new(radius))),
                                MeshMaterial2d(seg_mat.clone()),
                                Transform::from_xyz(p.x, p.y, 5.9),
                                Pickable::IGNORE,
                            ))
                            .id();
                        new_children.push(child);
                        local.segments[idx].child_entities.push(child);
                    }
                    if !new_children.is_empty() {
                        commands.entity(parent).add_children(&new_children);
                    }
                }

                for (i, child) in local.segments[idx].child_entities.iter().enumerate() {
                    if let Some(p) = smoothed.get(i) {
                        commands
                            .entity(*child)
                            .insert(Transform::from_xyz(p.x, p.y, 5.9));
                    }
                }
            }
        }
    } else {
        // Mouse not pressed: if there is an active segment, mark it released
        if let Some(idx) = local.current_segment {
            local.segments[idx].released_timer = Some(TRAIL_TTL);
            println!("[mouse_trail] released segment {} ttl={}", idx, TRAIL_TTL);
            local.current_segment = None;
        }
        local.last_pos = None;
    }
}

/// Updates fade and despawn for all trail segments.
pub fn update_mouse_trail(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut local: Local<MouseTrailLocal>,
    time: Res<Time>,
    query: Query<(&MouseTrailLine, &MeshMaterial2d<ColorMaterial>)>,
) {
    let dt = time.delta_secs();

    // Collect indices to remove after iteration
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, seg) in local.segments.iter_mut().enumerate() {
        // If parent entity gone, mark removed
        if query.get(seg.parent).is_err() {
            to_remove.push(i);
            continue;
        }

        if let Some(timer) = seg.released_timer {
            // `timer` is remaining time; alpha goes from 1.0 -> 0.0
            let alpha = (timer / TRAIL_TTL).clamp(0.0, 1.0);

            if let Some(mat) = materials.get_mut(&seg.mat_handle) {
                mat.color.set_alpha(alpha);
            }

            if timer <= 0.0 {
                println!("[mouse_trail] despawning segment {}", i);
                // despawn children then parent
                for child in &seg.child_entities {
                    commands.entity(*child).despawn();
                }
                commands.entity(seg.parent).despawn();
                to_remove.push(i);
            } else {
                // subtract delta
                seg.released_timer = Some(timer - dt);
            }
        } else {
            // ensure full alpha while drawing
            if let Some(mat) = materials.get_mut(&seg.mat_handle) {
                mat.color.set_alpha(1.0);
            }
        }
    }

    // remove segments in reverse order to keep indices stable
    for idx in to_remove.into_iter().rev() {
        // adjust current_segment index if needed
        if let Some(ci) = local.current_segment {
            if idx < ci {
                local.current_segment = Some(ci - 1);
            } else if idx == ci {
                local.current_segment = None;
            }
        }
        local.segments.remove(idx);
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
