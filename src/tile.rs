use bevy::prelude::*;

use crate::hex_grid::HexGrid;
use crate::{intersection, DebugUIState};

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
) -> impl Bundle {
    let assets = tile_assets.get_assets(&tile_type);

    // ------------------------------------------------------------------------
    // Deterministic per-tile variation (no RNG)
    // ------------------------------------------------------------------------
    let s1 = ((q * 97 + r * 31) as f32).sin();
    let c1 = ((q * 41 - r * 83) as f32).cos();
    let s2 = ((q * 19 + r * 53) as f32).sin();
    let c2 = ((q * 73 - r * 17) as f32).cos();

    let off_a = Vec2::new(s1 * 10.0, c1 * 10.0);
    let off_b = Vec2::new(s2 * 9.0, c2 * 9.0);
    let off_c = Vec2::new((s1 + s2) * 6.0, (c1 + c2) * 6.0);
    let off_d = Vec2::new((s1 - c2) * 7.0, (c1 + s2) * 7.0);
    let off_e = Vec2::new((c1 - s2) * 8.0, (s1 + c2) * 8.0);
    let off_f = Vec2::new((c2 - s1) * 7.0, (s2 - c1) * 7.0);

    // “Scratch-off” direction: rough tiles skew one way; swept tiles are tighter.
    let base_angle = (s1 * 0.9 + c2 * 0.6) * 0.9; // radians-ish
    let a1 = base_angle + 0.25;
    let a2 = base_angle - 0.35;
    let a3 = base_angle + 0.95;
    let a4 = base_angle - 1.05;
    let a5 = base_angle + 1.55;
    let a6 = base_angle - 1.65;

    let is_swept = tile_type == TileType::MaintainSpeed;
    let is_rough = tile_type == TileType::SlowDown;
    let is_wall = tile_type == TileType::Wall;
    let is_goal = tile_type == TileType::Goal;

    // ------------------------------------------------------------------------
    // Style materials
    // ------------------------------------------------------------------------
    // Ice lighting mats
    let (top_light_mat, bottom_shadow_mat, inner_glow_mat) = if is_swept {
        (
            tile_assets.swept_top_light_material.clone(),
            tile_assets.swept_bottom_shadow_material.clone(),
            tile_assets.swept_inner_glow_material.clone(),
        )
    } else if is_rough {
        (
            tile_assets.rough_top_light_material.clone(),
            tile_assets.rough_bottom_shadow_material.clone(),
            tile_assets.rough_inner_glow_material.clone(),
        )
    } else {
        (
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
        )
    };

    // Wall emboss mats
    let (wall_inner_shadow, wall_inner_highlight, wall_edge_glint) = if is_wall {
        (
            tile_assets.wall_inner_shadow_material.clone(),
            tile_assets.wall_inner_highlight_material.clone(),
            tile_assets.wall_edge_glint_material.clone(),
        )
    } else {
        (
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
        )
    };

    // Goal black-hole mats
    let (goal_hole_outer_mat, goal_hole_inner_mat, goal_hole_ring_mat) = if is_goal {
        (
            tile_assets.goal_hole_outer_material.clone(),
            tile_assets.goal_hole_inner_material.clone(),
            tile_assets.goal_hole_ring_material.clone(),
        )
    } else {
        (
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
        )
    };

    // Scratch-off texture mats
    // - Swept: very subtle “polish” streaks only.
    // - Rough: obvious scratch-off scuffs + chips.
    // - Goal: none.
    let (sheen_mat, scuff_light_mat, scuff_dark_mat, chip_mat) = if is_goal {
        (
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
        )
    } else if is_swept {
        (
            tile_assets.sheen_material.clone(),
            tile_assets.swept_scuff_light_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
        )
    } else if is_rough {
        (
            tile_assets.none_material.clone(),
            tile_assets.rough_scuff_light_material.clone(),
            tile_assets.rough_scuff_dark_material.clone(),
            tile_assets.rough_chip_material.clone(),
        )
    } else {
        (
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
            tile_assets.none_material.clone(),
        )
    };

    (
        tile_type,
        Visibility::Visible,
        Transform::from_xyz(world_pos.x, world_pos.y, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_6)),
        children![
            // ----------------------------------------------------------------
            // BORDER
            // ----------------------------------------------------------------
            (
                Mesh2d(tile_assets.hex_border_mesh.clone()),
                MeshMaterial2d(tile_assets.border_material.clone()),
            ),
            // Wall edge glint (bevel)
            (
                Mesh2d(tile_assets.hex_border_mesh.clone()),
                MeshMaterial2d(wall_edge_glint),
                Transform::from_xyz(0.0, 0.0, 0.20),
            ),

            // ----------------------------------------------------------------
            // MAIN FILL
            // ----------------------------------------------------------------
            (
                TileFill,
                Mesh2d(tile_assets.hex_mesh.clone()),
                MeshMaterial2d(assets.material.clone()),
                Transform::from_xyz(0.0, 0.0, 1.00),
            ),

            // ----------------------------------------------------------------
            // WALL: towering / imposing (ABOVE fill)
            // ----------------------------------------------------------------
            (
                Mesh2d(tile_assets.hex_mesh.clone()),
                MeshMaterial2d(wall_inner_shadow),
                Transform::from_xyz(-1.9, -2.2, 1.06).with_scale(Vec3::splat(0.90)),
            ),
            (
                Mesh2d(tile_assets.hex_mesh.clone()),
                MeshMaterial2d(wall_inner_highlight),
                Transform::from_xyz(2.2, 1.9, 1.07).with_scale(Vec3::splat(0.88)),
            ),

            // ----------------------------------------------------------------
            // ICE: directional lighting (ABOVE fill)
            // ----------------------------------------------------------------
            (
                Mesh2d(tile_assets.hex_mesh.clone()),
                MeshMaterial2d(top_light_mat),
                Transform::from_xyz(0.0, 4.4, 1.02).with_scale(Vec3::splat(0.965)),
            ),
            (
                Mesh2d(tile_assets.hex_mesh.clone()),
                MeshMaterial2d(bottom_shadow_mat),
                Transform::from_xyz(0.0, -4.4, 1.01).with_scale(Vec3::splat(0.965)),
            ),
            (
                Mesh2d(tile_assets.hex_mesh.clone()),
                MeshMaterial2d(inner_glow_mat),
                Transform::from_xyz(0.0, 1.7, 1.03).with_scale(Vec3::splat(0.92)),
            ),

            // ----------------------------------------------------------------
            // SWEPT (white): smooth, polished sheen + tiny polish streaks
            // ----------------------------------------------------------------
            (
                Mesh2d(tile_assets.sheen_mesh.clone()),
                MeshMaterial2d(sheen_mat),
                Transform::from_xyz(0.0, 0.0, 1.15)
                    .with_rotation(Quat::from_rotation_z(base_angle))
                    .with_scale(Vec3::new(1.0, 1.0, 1.0)),
            ),
            (
                Mesh2d(tile_assets.streak_mesh_thin.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(off_c.x * 0.35, off_c.y * 0.35, 1.16)
                    .with_rotation(Quat::from_rotation_z(a1))
                    .with_scale(Vec3::new(0.8, 0.8, 1.0)),
            ),
            (
                Mesh2d(tile_assets.streak_mesh_thin.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(-off_d.x * 0.25, off_d.y * 0.25, 1.161)
                    .with_rotation(Quat::from_rotation_z(a2))
                    .with_scale(Vec3::new(0.7, 0.7, 1.0)),
            ),

            // ----------------------------------------------------------------
            // ROUGH (light blue): SCRATCH-OFF scuffs + chips
            // ----------------------------------------------------------------
            // Wide scuff smears (these read like scraped ice)
            (
                Mesh2d(tile_assets.smear_mesh.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(off_a.x * 0.45, off_a.y * 0.45, 1.17)
                    .with_rotation(Quat::from_rotation_z(a1))
                    .with_scale(Vec3::new(1.0, 1.0, 1.0)),
            ),
            (
                Mesh2d(tile_assets.smear_mesh.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(off_b.x * 0.35, off_b.y * 0.35, 1.171)
                    .with_rotation(Quat::from_rotation_z(a2))
                    .with_scale(Vec3::new(0.9, 0.9, 1.0)),
            ),
            (
                Mesh2d(tile_assets.smear_mesh.clone()),
                MeshMaterial2d(scuff_dark_mat.clone()),
                Transform::from_xyz(off_e.x * 0.30, off_e.y * 0.30, 1.172)
                    .with_rotation(Quat::from_rotation_z(a3))
                    .with_scale(Vec3::new(0.85, 0.85, 1.0)),
            ),

            // Thin scratch streaks (layered = "scratch-off" texture)
            (
                Mesh2d(tile_assets.streak_mesh_long.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(off_c.x * 0.60, off_c.y * 0.60, 1.18)
                    .with_rotation(Quat::from_rotation_z(a1)),
            ),
            (
                Mesh2d(tile_assets.streak_mesh_long.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(off_d.x * 0.55, off_d.y * 0.55, 1.181)
                    .with_rotation(Quat::from_rotation_z(a2))
                    .with_scale(Vec3::new(0.95, 1.0, 1.0)),
            ),
            (
                Mesh2d(tile_assets.streak_mesh_long.clone()),
                MeshMaterial2d(scuff_dark_mat.clone()),
                Transform::from_xyz(off_f.x * 0.50, off_f.y * 0.50, 1.182)
                    .with_rotation(Quat::from_rotation_z(a4))
                    .with_scale(Vec3::new(0.9, 1.0, 1.0)),
            ),
            (
                Mesh2d(tile_assets.streak_mesh_short.clone()),
                MeshMaterial2d(scuff_light_mat.clone()),
                Transform::from_xyz(-off_a.x * 0.35, off_a.y * 0.25, 1.183)
                    .with_rotation(Quat::from_rotation_z(a5))
                    .with_scale(Vec3::new(0.9, 1.0, 1.0)),
            ),
            (
                Mesh2d(tile_assets.streak_mesh_short.clone()),
                MeshMaterial2d(scuff_dark_mat.clone()),
                Transform::from_xyz(off_b.x * 0.20, -off_b.y * 0.30, 1.184)
                    .with_rotation(Quat::from_rotation_z(a6))
                    .with_scale(Vec3::new(0.85, 1.0, 1.0)),
            ),

            // “Chips” along edges (tiny rough flecks, not dots everywhere)
            (
                Mesh2d(tile_assets.chip_mesh.clone()),
                MeshMaterial2d(chip_mat.clone()),
                Transform::from_xyz(10.0, 4.0, 1.19)
                    .with_rotation(Quat::from_rotation_z(a2))
                    .with_scale(Vec3::splat(0.9)),
            ),
            (
                Mesh2d(tile_assets.chip_mesh.clone()),
                MeshMaterial2d(chip_mat.clone()),
                Transform::from_xyz(-9.0, -3.0, 1.191)
                    .with_rotation(Quat::from_rotation_z(a5))
                    .with_scale(Vec3::splat(0.85)),
            ),
            (
                Mesh2d(tile_assets.chip_mesh.clone()),
                MeshMaterial2d(chip_mat.clone()),
                Transform::from_xyz(5.0, -9.0, 1.192)
                    .with_rotation(Quat::from_rotation_z(a1))
                    .with_scale(Vec3::splat(0.8)),
            ),

            // ----------------------------------------------------------------
            // GOAL: black hole (no other overlays)
            // ----------------------------------------------------------------
            (
                Mesh2d(tile_assets.goal_hole_outer_mesh.clone()),
                MeshMaterial2d(goal_hole_outer_mat),
                Transform::from_xyz(0.0, 0.0, 1.30),
            ),
            (
                Mesh2d(tile_assets.goal_hole_inner_mesh.clone()),
                MeshMaterial2d(goal_hole_inner_mat),
                Transform::from_xyz(0.0, 0.0, 1.31),
            ),
            (
                Mesh2d(tile_assets.goal_hole_ring_mesh.clone()),
                MeshMaterial2d(goal_hole_ring_mat),
                Transform::from_xyz(0.0, 0.0, 1.32),
            ),

            // ----------------------------------------------------------------
            // Debug coordinate text
            // ----------------------------------------------------------------
            (
                TileCoordinateText,
                Visibility::Hidden,
                Text2d::new(format!("{},{}", q, r)),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::BLACK),
                Transform::from_xyz(0., 0., 2.0)
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_6)),
            )
        ],
    )
}

// ============================================================================
// Constants
// ============================================================================

pub const COLORS: [Color; 6] = [
    Color::srgb(240.0 / 255.0, 250.0 / 255.0, 255.0 / 255.0), // swept ice
    Color::srgb(40.0 / 255.0, 225.0 / 255.0, 255.0 / 255.0),  // rough ice
    Color::srgb(90.0 / 255.0, 255.0 / 255.0, 200.0 / 255.0),
    Color::srgb(15.0 / 255.0, 70.0 / 255.0, 120.0 / 255.0),  // wall
    Color::srgb(8.0 / 255.0, 16.0 / 255.0, 30.0 / 255.0),    // near black
    Color::srgb(255.0 / 255.0, 60.0 / 255.0, 90.0 / 255.0),  // goal
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
}

#[derive(Component)]
pub struct TileFill;

#[derive(Component)]
pub struct TileCoordinateText;

#[derive(Component, Debug)]
pub struct TileDragging {
    pub last_position: Vec2,
    pub distance_dragged: f32,
}

#[derive(Component)]
pub struct MouseHover;

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource)]
pub struct TileAssets {
    pub hex_mesh: Handle<Mesh>,
    pub hex_border_mesh: Handle<Mesh>,
    pub border_material: Handle<ColorMaterial>,
    pub line_material: Handle<ColorMaterial>,

    // Common invisible material
    pub none_material: Handle<ColorMaterial>,

    // Texture meshes
    pub sheen_mesh: Handle<Mesh>,
    pub smear_mesh: Handle<Mesh>,
    pub streak_mesh_long: Handle<Mesh>,
    pub streak_mesh_short: Handle<Mesh>,
    pub streak_mesh_thin: Handle<Mesh>,
    pub chip_mesh: Handle<Mesh>,

    // Swept (white) ice materials
    pub swept_top_light_material: Handle<ColorMaterial>,
    pub swept_bottom_shadow_material: Handle<ColorMaterial>,
    pub swept_inner_glow_material: Handle<ColorMaterial>,
    pub sheen_material: Handle<ColorMaterial>,
    pub swept_scuff_light_material: Handle<ColorMaterial>,

    // Rough (light blue) ice materials
    pub rough_top_light_material: Handle<ColorMaterial>,
    pub rough_bottom_shadow_material: Handle<ColorMaterial>,
    pub rough_inner_glow_material: Handle<ColorMaterial>,
    pub rough_scuff_light_material: Handle<ColorMaterial>,
    pub rough_scuff_dark_material: Handle<ColorMaterial>,
    pub rough_chip_material: Handle<ColorMaterial>,

    // Wall (dark blue) “towering”
    pub wall_inner_shadow_material: Handle<ColorMaterial>,
    pub wall_inner_highlight_material: Handle<ColorMaterial>,
    pub wall_edge_glint_material: Handle<ColorMaterial>,

    // Goal black-hole meshes/materials
    pub goal_hole_outer_mesh: Handle<Mesh>,
    pub goal_hole_inner_mesh: Handle<Mesh>,
    pub goal_hole_ring_mesh: Handle<Mesh>,
    pub goal_hole_outer_material: Handle<ColorMaterial>,
    pub goal_hole_inner_material: Handle<ColorMaterial>,
    pub goal_hole_ring_material: Handle<ColorMaterial>,

    pub wall: TileTypeAssets,
    pub maintain_speed: TileTypeAssets,
    pub slow_down: TileTypeAssets,
    pub turn_counterclockwise: TileTypeAssets,
    pub turn_clockwise: TileTypeAssets,
    pub goal: TileTypeAssets,
}

pub struct TileTypeAssets {
    pub material: Handle<ColorMaterial>,
    pub hover_material: Handle<ColorMaterial>,
}

impl TileAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>, hex_grid: &HexGrid) -> Self {
        let border_thickness = 1.0;

        // Texture meshes:
        // - sheen: long thin highlight
        // - smear: wide “scraped” patch
        // - streaks: thin scratches
        // - chip: tiny irregular fleck (rectangle works fine)
        let sheen_mesh = meshes.add(Rectangle::new(hex_grid.hex_radius * 1.15, 8.0));
        let smear_mesh = meshes.add(Rectangle::new(hex_grid.hex_radius * 0.95, 18.0));
        let streak_mesh_long = meshes.add(Rectangle::new(hex_grid.hex_radius * 1.05, 3.2));
        let streak_mesh_short = meshes.add(Rectangle::new(hex_grid.hex_radius * 0.70, 2.8));
        let streak_mesh_thin = meshes.add(Rectangle::new(hex_grid.hex_radius * 0.70, 2.0));
        let chip_mesh = meshes.add(Rectangle::new(9.0, 2.2));

        // Goal hole meshes
        let goal_hole_outer_mesh = meshes.add(Circle::new(hex_grid.hex_radius * 0.56));
        let goal_hole_inner_mesh = meshes.add(Circle::new(hex_grid.hex_radius * 0.28));
        let goal_hole_ring_mesh = meshes.add(Circle::new(hex_grid.hex_radius * 0.40));

        // Materials (invisible)
        let none_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.0));

        // Border + line
        let border_material = materials.add(COLORS[4]);
        let line_material = materials.add(COLORS[5]);

        // Swept ice: smooth, mostly lighting + sheen
        let swept_top_light_material = materials.add(Color::srgba(1.0, 1.0, 1.0, 0.10));
        let swept_bottom_shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.08));
        let swept_inner_glow_material = materials.add(Color::srgba(0.92, 0.98, 1.0, 0.10));
        let sheen_material = materials.add(Color::srgba(0.70, 0.90, 1.0, 0.18)); // a visible polished streak
        let swept_scuff_light_material = materials.add(Color::srgba(0.85, 0.95, 1.0, 0.10)); // tiny polish streaks

        // Rough ice: stronger shading + scratch-off scuffs
        let rough_top_light_material = materials.add(Color::srgba(1.0, 1.0, 1.0, 0.06));
        let rough_bottom_shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.24));
        let rough_inner_glow_material = materials.add(Color::srgba(0.25, 0.60, 0.90, 0.08));

        // Scuffs: light and dark layers
        let rough_scuff_light_material = materials.add(Color::srgba(0.85, 0.97, 1.0, 0.18));
        let rough_scuff_dark_material = materials.add(Color::srgba(0.03, 0.05, 0.07, 0.18));
        let rough_chip_material = materials.add(Color::srgba(0.02, 0.03, 0.04, 0.32)); // edge chips

        // Wall: towering via visible inner shadow/highlight + bevel glint
        let wall_inner_shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.35));
        let wall_inner_highlight_material = materials.add(Color::srgba(0.60, 0.85, 1.00, 0.16));
        let wall_edge_glint_material = materials.add(Color::srgba(0.65, 0.90, 1.0, 0.18));

        // Goal "black hole"
        let goal_hole_outer_material = materials.add(Color::srgba(0.12, 0.00, 0.06, 0.60));
        let goal_hole_inner_material = materials.add(Color::srgba(0.00, 0.00, 0.00, 0.90));
        let goal_hole_ring_material = materials.add(Color::srgba(1.00, 0.55, 0.70, 0.20));

        TileAssets {
            hex_mesh: meshes.add(RegularPolygon::new(hex_grid.hex_radius - border_thickness, 6)),
            hex_border_mesh: meshes.add(RegularPolygon::new(hex_grid.hex_radius, 6)),
            border_material,
            line_material,

            none_material,

            sheen_mesh,
            smear_mesh,
            streak_mesh_long,
            streak_mesh_short,
            streak_mesh_thin,
            chip_mesh,

            swept_top_light_material,
            swept_bottom_shadow_material,
            swept_inner_glow_material,
            sheen_material,
            swept_scuff_light_material,

            rough_top_light_material,
            rough_bottom_shadow_material,
            rough_inner_glow_material,
            rough_scuff_light_material,
            rough_scuff_dark_material,
            rough_chip_material,

            wall_inner_shadow_material,
            wall_inner_highlight_material,
            wall_edge_glint_material,

            goal_hole_outer_mesh,
            goal_hole_inner_mesh,
            goal_hole_ring_mesh,
            goal_hole_outer_material,
            goal_hole_inner_material,
            goal_hole_ring_material,

            wall: TileTypeAssets {
                material: materials.add(COLORS[3]),
                hover_material: materials.add(COLORS[3].with_alpha(0.85)),
            },
            maintain_speed: TileTypeAssets {
                material: materials.add(COLORS[0]),
                hover_material: materials.add(COLORS[0].with_alpha(0.92)),
            },
            slow_down: TileTypeAssets {
                material: materials.add(COLORS[1]),
                hover_material: materials.add(COLORS[1].with_alpha(0.92)),
            },
            turn_counterclockwise: TileTypeAssets {
                material: materials.add(COLORS[2]),
                hover_material: materials.add(COLORS[2].with_alpha(0.85)),
            },
            turn_clockwise: TileTypeAssets {
                material: materials.add(COLORS[4]),
                hover_material: materials.add(COLORS[4].with_alpha(0.85)),
            },
            goal: TileTypeAssets {
                material: materials.add(COLORS[5]),
                hover_material: materials.add(COLORS[5].with_alpha(0.92)),
            },
        }
    }

    pub fn get_assets(&self, tile_type: &TileType) -> &TileTypeAssets {
        match tile_type {
            TileType::Wall => &self.wall,
            TileType::MaintainSpeed => &self.maintain_speed,
            TileType::SlowDown => &self.slow_down,
            TileType::TurnCounterclockwise => &self.turn_counterclockwise,
            TileType::TurnClockwise => &self.turn_clockwise,
            TileType::Goal => &self.goal,
        }
    }
}

// ============================================================================
// Systems
// ============================================================================

pub fn change_tile_type(
    input: Res<ButtonInput<KeyCode>>,
    mut tile_type: Single<&mut TileType, With<MouseHover>>,
) {
    if **tile_type == TileType::Goal {
        return;
    }
    if input.just_pressed(KeyCode::KeyW) {
        **tile_type = TileType::MaintainSpeed;
    }
    if input.just_pressed(KeyCode::KeyA) {
        **tile_type = TileType::TurnClockwise;
    }
    if input.just_pressed(KeyCode::KeyD) {
        **tile_type = TileType::TurnCounterclockwise;
    }
    if input.just_pressed(KeyCode::KeyS) {
        **tile_type = TileType::SlowDown;
    }
}

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

pub fn update_tile_material(
    tile_query: Query<(Entity, &TileType, Option<&MouseHover>)>,
    children_query: Query<&Children>,
    tile_assets: Res<TileAssets>,
    mut fill_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<TileFill>>,
) {
    for (entity, tile_type, mouse_hover) in tile_query {
        if *tile_type == TileType::Wall || *tile_type == TileType::Goal {
            continue;
        }
        let Ok(children) = children_query.get(entity) else {
            continue;
        };
        let assets = tile_assets.get_assets(tile_type);
        let material = if mouse_hover.is_some() {
            &assets.hover_material
        } else {
            &assets.material
        };
        for child in children.iter() {
            if let Ok(mut mesh_material) = fill_query.get_mut(child) {
                mesh_material.0 = material.clone();
            }
        }
    }
}

pub fn update_tile_type(debug_ui_state: Res<DebugUIState>, tiles: Query<(&TileDragging, &mut TileType)>) {
    for (tile_dragging, mut tile_type) in tiles {
        if tile_dragging.distance_dragged > debug_ui_state.min_sweep_distance {
            *tile_type = TileType::MaintainSpeed;
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
) {
    if let Ok(Some(mut tile_dragging)) = tile_dragging_q.get_mut(drag_enter.entity) {
        tile_dragging.last_position = drag_enter.pointer_location.position;
    } else {
        commands.entity(drag_enter.entity).insert(TileDragging {
            last_position: drag_enter.pointer_location.position,
            distance_dragged: 0.0,
        });
    }
}

pub fn on_tile_dragging(
    drag: On<Pointer<Drag>>,
    mut tile: Single<(&mut TileDragging, &TileType), With<MouseHover>>,
) {
    if *tile.1 == TileType::Goal || *tile.1 == TileType::Wall {
        return;
    }
    tile.0.distance_dragged += (drag.pointer_location.position - tile.0.last_position).length();
    tile.0.last_position = drag.pointer_location.position;
}

// ============================================================================
// Physics
// ============================================================================

const HEX_EDGE_NORMALS: [Vec2; 6] = [
    Vec2::new(0.8660254, 0.5),
    Vec2::new(0.0, 1.0),
    Vec2::new(-0.8660254, 0.5),
    Vec2::new(-0.8660254, -0.5),
    Vec2::new(0.0, -1.0),
    Vec2::new(0.8660254, -0.5),
];

fn hex_edge_normal(relative_pos: Vec2) -> Vec2 {
    let angle = relative_pos.y.atan2(relative_pos.x);
    let angle = if angle < 0.0 {
        angle + std::f32::consts::TAU
    } else {
        angle
    };
    let sector = ((angle / std::f32::consts::FRAC_PI_3) as usize).min(5);
    HEX_EDGE_NORMALS[sector]
}

pub fn compute_tile_effects(
    stone_pos: Vec2,
    velocity: &crate::stone::Velocity,
    tiles: &[(&TileType, Vec2)],
    hex_grid: &HexGrid,
    drag_coefficient: f32,
    stone_radius: f32,
    slow_down_factor: f32,
    rotation_factor: f32,
) -> crate::stone::Velocity {
    let mut new_velocity = velocity.0;

    let mut rotation_angle: f32 = 0.0;
    let mut total_drag: f32 = 0.0;

    for &(tile_type, tile_world_pos) in tiles {
        let ratio = intersection::ratio_circle_area_inside_hexagon(
            stone_pos,
            stone_radius,
            tile_world_pos,
            hex_grid.hex_radius - 2.,
            60,
        );
        if ratio < 0.01 {
            continue;
        }

        match tile_type {
            TileType::Wall => {
                let wall_normal = hex_edge_normal(stone_pos - tile_world_pos);
                let dot = new_velocity.dot(wall_normal);
                if dot < 0.0 {
                    let original_speed = new_velocity.length();
                    new_velocity -= 2.0 * dot * wall_normal;
                    let new_speed = new_velocity.length();
                    if new_speed > 1e-10 {
                        new_velocity *= original_speed / new_speed;
                    }
                }
            }
            TileType::MaintainSpeed => {
                total_drag += drag_coefficient * ratio;
            }
            TileType::SlowDown => {
                total_drag += drag_coefficient * ratio * slow_down_factor;
            }
            TileType::TurnCounterclockwise => {
                rotation_angle += rotation_factor * ratio;
                total_drag += drag_coefficient * ratio;
            }
            TileType::TurnClockwise => {
                rotation_angle -= rotation_factor * ratio;
                total_drag += drag_coefficient * ratio;
            }
            TileType::Goal => {
                let to_center = tile_world_pos - stone_pos;
                let distance = to_center.length();
                if distance > 1e-10 {
                    let direction = to_center / distance;
                    let pull_strength = 0.5 * ratio;
                    new_velocity += direction * pull_strength;
                }
                total_drag += drag_coefficient * slow_down_factor * ratio;
            }
        }
    }

    if rotation_angle.abs() > 1e-10 {
        let (sin_angle, cos_angle) = rotation_angle.sin_cos();
        new_velocity = Vec2::new(
            new_velocity.x * cos_angle - new_velocity.y * sin_angle,
            new_velocity.x * sin_angle + new_velocity.y * cos_angle,
        );
    }

    if total_drag > 0.0 {
        let drag_factor = (1.0 - total_drag).max(0.0);
        new_velocity *= drag_factor;
    }

    crate::stone::Velocity(new_velocity)
}
