use std::{collections::HashMap, fmt::Display, slice::Iter};

use bevy::prelude::*;

use crate::tile::{
    ScratchOffMaterial, TileAssets, TileType, on_pointer_out, on_pointer_over, on_tile_drag_enter,
    on_tile_dragging, tile,
};

/// Component for the hex grid entity.
/// Tiles are spawned as children of this entity.
#[derive(Component, Clone)]
pub struct HexGrid {
    pub hex_radius: f32,
    pub horiz_spacing: f32,
    pub vert_spacing: f32,
    pub cols: i32,
    pub rows: i32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub level: Level,
}

impl HexGrid {
    pub fn new(hex_radius: f32, level: &Level) -> Self {
        let cols = level.grid.keys().map(|coord| coord.q).max().unwrap_or(0) + 1;
        let rows = level.grid.keys().map(|coord| coord.r).max().unwrap_or(0) + 1;
        let horiz_spacing = hex_radius * 1.5;
        let vert_spacing = hex_radius * 3.0_f32.sqrt();
        let offset_x = -(cols as f32 * horiz_spacing) / 2.0;
        let offset_y = -(rows as f32 * vert_spacing) / 2.0;

        Self {
            hex_radius,
            horiz_spacing,
            vert_spacing,
            cols,
            rows,
            offset_x,
            offset_y,
            level: level.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct HexCoordinate {
    pub q: i32,
    pub r: i32,
}

/// Converts hex grid coordinates to world position for flat-top hexagons
pub fn hex_to_world(hex_coord: &HexCoordinate, hex_grid: &HexGrid) -> Vec2 {
    let x = hex_grid.offset_x + hex_coord.q as f32 * hex_grid.horiz_spacing;
    let y_offset = if hex_coord.q % 2 == 1 {
        hex_grid.vert_spacing / 2.0
    } else {
        0.0
    };
    let y = hex_grid.offset_y
        + (hex_grid.rows - 1 - hex_coord.r) as f32 * hex_grid.vert_spacing
        + y_offset;

    Vec2::new(x, y)
}

pub fn spawn_hex_grid(
    commands: &mut Commands,
    grid: &HexGrid,
    tile_assets: &TileAssets,
    scratch_materials: &mut Assets<ScratchOffMaterial>,
) -> Entity {
    let mut tile_entities = Vec::new();

    for q in 0..grid.cols {
        for r in 0..grid.rows {
            let world_pos = hex_to_world(&HexCoordinate { q, r }, grid);
            if let Some(tile_type) = grid.level.grid.get(&HexCoordinate { q, r }) {
                let tile_id = commands
                    .spawn(tile(
                        tile_type.clone(),
                        world_pos,
                        q,
                        r,
                        tile_assets,
                        scratch_materials,
                    ))
                    .observe(on_pointer_over)
                    .observe(on_pointer_out)
                    .observe(on_tile_dragging)
                    .observe(on_tile_drag_enter)
                    .id();
                tile_entities.push(tile_id);
            }
        }
    }

    commands
        .spawn((
            Visibility::Visible,
            Transform::from_xyz(0., 0., 0.),
            grid.clone(),
        ))
        .add_children(&tile_entities)
        .id()
}

#[derive(Clone, PartialEq, Debug)]
pub enum Facing {
    Up,
    UpRight,
    DownRight,
    Down,
    DownLeft,
    UpLeft,
}

impl Facing {
    pub fn iterator() -> Iter<'static, Facing> {
        static DIRECTIONS: [Facing; 6] = [
            Facing::Up,
            Facing::UpRight,
            Facing::DownRight,
            Facing::Down,
            Facing::DownLeft,
            Facing::UpLeft,
        ];
        DIRECTIONS.iter()
    }
}

impl Display for Facing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Level {
    pub grid: HashMap<HexCoordinate, TileType>,
    pub goal_coordinate: HexCoordinate,
    pub stone_configs: Vec<StoneConfig>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StoneConfig {
    pub velocity_magnitude: f32,
    pub start_coordinate: HexCoordinate,
    pub facing: Facing,
}

pub fn get_initial_stone_velocity(facing: &Facing, stone_velocity_magnitude: &f32) -> Vec2 {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_6};
    let angle = match facing {
        Facing::Up => FRAC_PI_2,                    // 90° - straight up
        Facing::UpRight => FRAC_PI_6,               // 30° - up and right
        Facing::DownRight => -FRAC_PI_6,            // -30° - down and right
        Facing::Down => -FRAC_PI_2,                 // -90° - straight down
        Facing::DownLeft => -FRAC_PI_2 - FRAC_PI_3, // -120° - down and left
        Facing::UpLeft => FRAC_PI_2 + FRAC_PI_3,    // 120° - up and left
    };
    Vec2::from_angle(angle) * *stone_velocity_magnitude
}

pub fn get_level() -> Level {
    let goal_coordinate = HexCoordinate { q: 7, r: 4 };
    let start_coordinate = HexCoordinate { q: 1, r: 1 };

    let grid = HashMap::from([
        (HexCoordinate { q: 0, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 6, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 4 }, TileType::Wall),
        (HexCoordinate { q: 6, r: 4 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 8, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 8, r: 4 }, TileType::Wall),
        //
        (start_coordinate.clone(), TileType::MaintainSpeed),
        (HexCoordinate { q: 2, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 3, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 6, r: 3 }, TileType::SlowDown),
        (goal_coordinate.clone(), TileType::Goal),
    ]);

    Level {
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate: HexCoordinate { q: 1, r: 1 },
            velocity_magnitude: 500.0,
            facing: Facing::DownRight,
        }],
    }
}

// pub fn get_level() -> Level {
//     let goal_coordinate = HexCoordinate { q: 4, r: 2 };
//     let start_coordinate = HexCoordinate { q: 4, r: 3 };

//     let grid = HashMap::from([
//         (HexCoordinate { q: 0, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 1 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 2 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 3 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 4 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 5 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 6 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 7 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 8 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 9 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 1, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 2, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 3, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 4, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 5, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 6, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 7, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 8, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 9, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 0 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 1 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 2 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 3 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 4 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 5 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 6 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 7 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 8 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 9 }, TileType::Wall),
//         (HexCoordinate { q: 10, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 0, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 1, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 2, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 3, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 4, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 5, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 6, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 7, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 8, r: 10 }, TileType::Wall),
//         (HexCoordinate { q: 9, r: 10 }, TileType::Wall),
//         //
//         (HexCoordinate { q: 1, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 1, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 2, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 2, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 3, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 3, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 4, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 4, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 5, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 5, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 6, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 6, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 7, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 7, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 8, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 8, r: 9 }, TileType::SlowDown),
//         //
//         (HexCoordinate { q: 9, r: 1 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 2 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 3 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 4 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 5 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 6 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 7 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 8 }, TileType::SlowDown),
//         (HexCoordinate { q: 9, r: 9 }, TileType::SlowDown),
//         //
//         (start_coordinate.clone(), TileType::MaintainSpeed),
//         (goal_coordinate.clone(), TileType::Goal),
//     ]);

//     Level {
//         grid,
//         goal_coordinate: goal_coordinate.clone(),
//         stone_configs: vec![
//             StoneConfig {
//                 velocity_magnitude: 250.0,
//                 start_coordinate,
//                 facing: Facing::Up,
//             },
//             StoneConfig {
//                 velocity_magnitude: 0.0,
//                 start_coordinate: goal_coordinate,
//                 facing: Facing::DownRight,
//             },
//         ],
//     }
// }
