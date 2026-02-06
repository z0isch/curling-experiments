use bevy::{
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    sprite_render::Material2dPlugin,
};
use bevy_egui::EguiPrimaryContextPass;
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{asset_tracking::LoadResource, confetti::ConfettiMaterial, tile::IsGoal};

use crate::{
    PausableSystems,
    debug_ui::{DebugUIState, StoneUIConfig, debug_ui, on_debug_ui_level_change},
    fire_trail::{spawn_fire_trail, update_fire_trail},
    hex_grid::{HexGrid, spawn_hex_grid},
    level::{CurrentLevel, Level, get_initial_stone_velocity, get_level},
    screens::Screen,
    stone::{
        Stone, Velocity, apply_stone_collision, apply_tile_velocity_effects, resolve_collision,
        stone, update_stone_position,
    },
    tile::{
        CurrentDragTileType, ScratchOffMaterial, TileAssets, TileDragging, TileType,
        compute_tile_effects, toggle_tile_coordinates, update_tile_material,
    },
    ui,
};

#[derive(Component)]
struct StoneMoveLine;

#[derive(Event)]
pub struct LevelComplete;

#[derive(Event)]
pub struct StoneStopped;

#[derive(Resource)]
pub struct OnLevel(pub Level);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainUpdateSystems;

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Initial,
    Countdown,
    Playing,
}

#[derive(Component)]
pub struct Celebration;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<ScratchOffMaterial>::default())
        .add_plugins(Material2dPlugin::<ConfettiMaterial>::default())
        .add_plugins(ui::plugin);

    app.init_state::<GameState>();
    app.load_resource::<GameplayAssets>();
    app.add_systems(Startup, setup);
    app.add_systems(
        FixedUpdate,
        (
            apply_stone_collision,
            update_stone_position,
            apply_tile_velocity_effects,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameState::Playing))
            .in_set(PausableSystems),
    )
    .add_systems(
        Update,
        (spawn_fire_trail, update_fire_trail)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameState::Playing))
            .in_set(PausableSystems),
    )
    .add_systems(
        Update,
        (
            draw_move_line,
            toggle_tile_coordinates,
            update_tile_material,
            switch_broom,
            level_0_complete_check,
            celebrate,
            play_get_in_there,
        )
            .in_set(MainUpdateSystems)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    )
    .add_systems(
        Update,
        (restart_game_on_r_key_pressed, on_debug_ui_level_change)
            .after(MainUpdateSystems)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    )
    .add_systems(EguiPrimaryContextPass, debug_ui)
    .add_observer(on_level_complete);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameplayAssets {
    #[dependency]
    crowd: Handle<AudioSample>,
    #[dependency]
    get_in_there: Handle<AudioSample>,
}

impl FromWorld for GameplayAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            crowd: assets.load("audio/sfx/crowd.ogg"),
            get_in_there: assets.load("audio/sfx/get_in_there.ogg"),
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let current_level = CurrentLevel::default();
    let level = get_level(current_level);
    commands.insert_resource(OnLevel(level.clone()));

    let debug_ui_state = DebugUIState {
        hex_radius: 60.0,
        stone_radius: 15.0,
        min_sweep_distance: 250.0,
        drag_coefficient: 0.0036,
        slow_down_factor: 5.0,
        rotation_factor: 0.025,
        snap_distance: 40.0,
        snap_velocity: 40.0,
        current_level,
        speed_up_factor: 250.0,
        stone_configs: level
            .stone_configs
            .iter()
            .map(|sc| StoneUIConfig {
                velocity_magnitude: sc.velocity_magnitude,
                facing: sc.facing.clone(),
            })
            .collect(),
    };

    let grid = HexGrid::new(&level);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);
    commands.insert_resource(tile_assets);
    commands.insert_resource(debug_ui_state.clone());
    commands.insert_resource(CurrentDragTileType(TileType::MaintainSpeed));
}

pub fn spawn_game(
    mut commands: Commands,
    grid: Query<Entity, With<HexGrid>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
    current_drag_tile_type: ResMut<CurrentDragTileType>,
    on_level: Res<OnLevel>,
) {
    restart_game(
        &mut commands,
        grid,
        debug_ui_state,
        stone_query,
        meshes,
        materials,
        scratch_materials,
        current_drag_tile_type,
        Some(&on_level.0),
    );
}

#[derive(Resource)]
pub struct CelebrationTimer(Timer);

impl Default for CelebrationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3.0, TimerMode::Once))
    }
}

fn celebrate(
    mut commands: Commands,
    time: Res<Time>,
    celebration_query: Query<(Entity, &MeshMaterial2d<ConfettiMaterial>), With<Celebration>>,
    mut on_level: ResMut<OnLevel>,
    grid: Query<Entity, With<HexGrid>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
    mut confetti_materials: ResMut<Assets<ConfettiMaterial>>,
    current_drag_tile_type: ResMut<CurrentDragTileType>,
    mut celebration_timer: Local<CelebrationTimer>,
) {
    if let Some((celebration_entity, material_handle)) = celebration_query.iter().next() {
        if let Some(material) = confetti_materials.get_mut(&material_handle.0) {
            material.params.x += time.delta_secs();
        }
        celebration_timer.0.tick(time.delta());
        if celebration_timer.0.is_finished()
            && let Some(next_level) = CurrentLevel::iterator()
                .skip_while(|&level| level != &on_level.0.current_level)
                .nth(1)
        {
            on_level.0 = get_level(*next_level).clone();

            restart_game(
                &mut commands,
                grid,
                debug_ui_state,
                stone_query,
                meshes,
                materials,
                scratch_materials,
                current_drag_tile_type,
                Some(&get_level(*next_level)),
            );
            commands.entity(celebration_entity).despawn();
            celebration_timer.0.reset();
        }
    }
}

fn on_level_complete(
    _event: On<LevelComplete>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    gameplay_assets: Res<GameplayAssets>,
    mut confetti_materials: ResMut<Assets<ConfettiMaterial>>,
) {
    commands.spawn((
        Celebration,
        Mesh2d(meshes.add(Rectangle::new(5000.0, 5000.0))),
        MeshMaterial2d(confetti_materials.add(ConfettiMaterial {
            params: Vec4::new(0.0, 0.0, 0.0, 0.0),
        })),
        Transform::from_xyz(0.0, 0.0, 100.0), // High Z-index
    ));
    commands.spawn(SamplePlayer::new(gameplay_assets.crowd.clone()));
}

pub fn switch_broom(
    input: Res<ButtonInput<KeyCode>>,
    mut current_drag_tile_type: ResMut<CurrentDragTileType>,
) {
    if input.just_pressed(KeyCode::Digit1) {
        *current_drag_tile_type = CurrentDragTileType(TileType::MaintainSpeed);
    }
    if input.just_pressed(KeyCode::Digit2) {
        *current_drag_tile_type = CurrentDragTileType(TileType::TurnCounterclockwise);
    }
    if input.just_pressed(KeyCode::Digit3) {
        *current_drag_tile_type = CurrentDragTileType(TileType::TurnClockwise);
    }
}

fn restart_game_on_r_key_pressed(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    grid: Query<Entity, With<HexGrid>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
    current_drag_tile_type: ResMut<CurrentDragTileType>,
    on_level: Res<OnLevel>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        restart_game(
            &mut commands,
            grid,
            debug_ui_state,
            stone_query,
            meshes,
            materials,
            scratch_materials,
            current_drag_tile_type,
            Some(&on_level.0),
        );
    }
}
pub fn restart_game(
    commands: &mut Commands,
    grid: Query<Entity, With<HexGrid>>,
    debug_ui_state: Res<DebugUIState>,
    stone_query: Query<Entity, With<Stone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
    mut current_drag_tile_type: ResMut<CurrentDragTileType>,
    level: Option<&Level>,
) {
    *current_drag_tile_type = CurrentDragTileType(TileType::MaintainSpeed);
    let debug_level = get_level(debug_ui_state.current_level);
    let level = level.unwrap_or(&debug_level);

    for grid_entity in grid {
        commands.entity(grid_entity).despawn();
    }

    let grid = HexGrid::new(level);
    let tile_assets = TileAssets::new(&mut meshes, &mut materials, &grid);
    spawn_hex_grid(
        commands,
        &grid,
        &tile_assets,
        &debug_ui_state,
        &mut scratch_materials,
    );
    for stone_entity in stone_query {
        commands.entity(stone_entity).despawn();
    }
    for stone_config in level.stone_configs.iter() {
        commands.spawn((
            DespawnOnExit(Screen::Gameplay),
            stone(
                &mut meshes,
                &mut materials,
                &grid,
                &stone_config.start_coordinate,
                get_initial_stone_velocity(&stone_config.facing, &stone_config.velocity_magnitude),
                &debug_ui_state.stone_radius,
            ),
        ));
    }
    commands.set_state(GameState::Countdown);
}

fn draw_move_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    tile_assets: Res<TileAssets>,
    grid: Single<&HexGrid>,
    debug_ui_state: Res<DebugUIState>,
    stones: Query<(&Stone, &Velocity, &Transform)>,
    tiles: Query<(&Transform, &TileDragging), Without<Stone>>,
    lines: Query<Entity, With<StoneMoveLine>>,
    fixed_time: Res<Time<Fixed>>,
) {
    for l in &lines {
        commands.entity(l).despawn();
    }

    // Collect tile data for trajectory simulation (including dragging state)
    let tile_data: Vec<_> = tiles
        .iter()
        .map(|(transform, tile_dragging)| {
            let position = transform.translation.truncate();
            (position, tile_dragging)
        })
        .collect();

    // Collect all stone data for multi-stone simulation
    let stone_data: Vec<_> = stones
        .iter()
        .map(|(stone, velocity, transform)| {
            (
                transform.translation.truncate(),
                velocity.clone(),
                stone.radius,
            )
        })
        .collect();

    // Simulate physics forward to predict trajectory for all stones together
    let trajectories = simulate_trajectories(
        &stone_data,
        &tile_data,
        *grid,
        debug_ui_state.drag_coefficient,
        fixed_time.delta_secs(),
        debug_ui_state.slow_down_factor,
        debug_ui_state.rotation_factor,
        debug_ui_state.speed_up_factor,
    );

    for trajectory in trajectories {
        if let Some(mesh) = create_tapered_line_mesh(&trajectory, 6.0, 1.0) {
            commands.spawn((
                DespawnOnExit(Screen::Gameplay),
                StoneMoveLine,
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(tile_assets.line_material.clone()),
                Transform::from_xyz(0., 0., 2.0),
            ));
        }
    }
}

/// Creates a tapered line mesh that starts thick and thins out along the trajectory.
fn create_tapered_line_mesh(points: &[Vec2], start_width: f32, end_width: f32) -> Option<Mesh> {
    if points.len() < 2 {
        return None;
    }

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(points.len() * 2);
    let mut indices: Vec<u32> = Vec::with_capacity((points.len() - 1) * 6);

    let total_points = points.len();

    for (i, point) in points.iter().enumerate() {
        // Calculate the direction at this point
        let direction = if i == 0 {
            (points[1] - points[0]).normalize_or_zero()
        } else if i == total_points - 1 {
            (points[i] - points[i - 1]).normalize_or_zero()
        } else {
            // Average direction from neighbors for smoother curves
            ((points[i] - points[i - 1]).normalize_or_zero()
                + (points[i + 1] - points[i]).normalize_or_zero())
            .normalize_or_zero()
        };

        // Perpendicular direction (rotate 90 degrees)
        let perpendicular = Vec2::new(-direction.y, direction.x);

        // Interpolate width from start to end (using sqrt for faster tapering)
        let t = (i as f32 / (total_points - 1) as f32).sqrt();
        let half_width = (start_width * (1.0 - t) + end_width * t) / 2.0;

        // Create two vertices on either side of the line
        let left = *point + perpendicular * half_width;
        let right = *point - perpendicular * half_width;

        positions.push([left.x, left.y, 0.0]);
        positions.push([right.x, right.y, 0.0]);

        // Create triangles connecting to previous segment
        if i > 0 {
            let base = (i as u32 - 1) * 2;
            // Two triangles forming a quad
            indices.push(base); // prev left
            indices.push(base + 1); // prev right
            indices.push(base + 2); // curr left

            indices.push(base + 1); // prev right
            indices.push(base + 3); // curr right
            indices.push(base + 2); // curr left
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));

    Some(mesh)
}

/// Simulates all stones' trajectories by forward-integrating physics.
///
/// **Important**: The order of operations must match the FixedUpdate system chain:
/// 1. apply_stone_collision (handle collisions)
/// 2. update_stone_position (move)
/// 3. apply_tile_velocity_effects (update velocity)
fn simulate_trajectories(
    stone_data: &[(Vec2, Velocity, f32)], // (position, velocity, radius)
    tile_data: &[(Vec2, &TileDragging)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    fixed_dt: f32,
    slow_down_factor: f32,
    rotation_factor: f32,
    speed_up_factor: f32,
) -> Vec<Vec<Vec2>> {
    const MIN_VELOCITY: f32 = 1.0; // Stop when velocity is very low
    const LINE_SEGMENT_SAMPLES: usize = 3;

    // Initialize simulation state for each stone
    let mut stones: Vec<_> = stone_data
        .iter()
        .map(|(pos, vel, radius)| (*pos, vel.clone(), *radius))
        .collect();

    let mut trajectories: Vec<Vec<Vec2>> = stones.iter().map(|(pos, _, _)| vec![*pos]).collect();

    let steps = 10000;
    for i in 0..steps {
        // Check if all stones have stopped
        let all_stopped = stones
            .iter()
            .all(|(_, vel, _)| vel.0.length_squared() < MIN_VELOCITY * MIN_VELOCITY);
        if all_stopped {
            break;
        }

        // Step 1: Apply stone collisions (matches apply_stone_collision)
        for j in 0..stones.len() {
            for k in (j + 1)..stones.len() {
                let (pos1, vel1, radius1) = &stones[j];
                let (pos2, vel2, radius2) = &stones[k];

                if let Some((new_vel1, new_vel2)) =
                    resolve_collision(*pos1, vel1, *radius1, *pos2, vel2, *radius2)
                {
                    stones[j].1 = new_vel1;
                    stones[k].1 = new_vel2;
                }
            }
        }

        // Step 2: Move positions (matches update_stone_position)
        for (pos, vel, _) in &mut stones {
            *pos += vel.0 * fixed_dt;
        }

        // Record trajectory points
        if i % LINE_SEGMENT_SAMPLES == 0 {
            for (idx, (pos, _, _)) in stones.iter().enumerate() {
                trajectories[idx].push(*pos);
            }
        }

        // Step 3: Update velocities based on new positions (matches apply_tile_velocity_effects)
        for (pos, vel, radius) in &mut stones {
            *vel = compute_tile_effects(
                *pos,
                vel,
                tile_data,
                hex_grid,
                drag_coefficient,
                *radius,
                slow_down_factor,
                rotation_factor,
                speed_up_factor,
            );
        }
    }

    // Always include the final positions
    for (idx, (pos, _, _)) in stones.iter().enumerate() {
        if trajectories[idx].last() != Some(pos) {
            trajectories[idx].push(*pos);
        }
    }

    trajectories
}

fn level_0_complete_check(
    mut commands: Commands,
    on_level: Res<OnLevel>,
    tile_query: Query<&TileDragging>,
    debug_ui_state: Res<DebugUIState>,
    mut has_reached_goal: Local<bool>,
) {
    if (on_level.0.current_level == CurrentLevel::Level0)
        && tile_query.iter().all(|tile_dragging| {
            *tile_dragging
                .distance_dragged
                .get(&TileType::MaintainSpeed)
                .unwrap_or(&0.0)
                + 2.0
                >= debug_ui_state.min_sweep_distance
        })
        && !*has_reached_goal
    {
        commands.trigger(LevelComplete);
        *has_reached_goal = true;
    }
}

#[derive(Component)]
pub struct PlayedGetInThere;

fn play_get_in_there(
    mut commands: Commands,
    gameplay_assets: Res<GameplayAssets>,
    stone_query: Single<(Entity, &Transform, &Velocity), (With<Stone>, Without<PlayedGetInThere>)>,
    goal_query: Single<&Transform, (With<IsGoal>, Without<Stone>)>,
    debug_ui_state: Res<DebugUIState>,
) {
    let min_dist_from_snap = 80.0;
    let min_velocity = 40.0;
    let distance_from_goal_squared = stone_query
        .1
        .translation
        .truncate()
        .distance_squared(goal_query.translation.truncate());
    let velocity_squared = stone_query.2.0.length_squared();
    let inside_goal_tile =
        distance_from_goal_squared <= debug_ui_state.hex_radius * debug_ui_state.hex_radius;
    if distance_from_goal_squared
        < (min_dist_from_snap * min_dist_from_snap)
            + (debug_ui_state.snap_distance * debug_ui_state.snap_distance)
        && velocity_squared < min_velocity * min_velocity
        && !inside_goal_tile
    {
        commands.entity(stone_query.0).insert(PlayedGetInThere);
        commands.spawn(SamplePlayer::new(gameplay_assets.get_in_there.clone()));
    }
}
