use std::{collections::HashMap, fmt::Display, slice::Iter};

use bevy::prelude::*;

use crate::{hex_grid::HexCoordinate, tile::TileType};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CurrentLevel {
    #[default]
    Level1,
    Level2,
}

impl CurrentLevel {
    pub fn iterator() -> Iter<'static, CurrentLevel> {
        static LEVELS: [CurrentLevel; 2] = [CurrentLevel::Level1, CurrentLevel::Level2];
        LEVELS.iter()
    }
}

impl Display for CurrentLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentLevel::Level1 => write!(f, "Level 1"),
            CurrentLevel::Level2 => write!(f, "Level 2"),
        }
    }
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

pub fn get_level(current_level: CurrentLevel) -> Level {
    match current_level {
        CurrentLevel::Level1 => get_level1(),
        CurrentLevel::Level2 => get_level2(),
    }
}

fn get_level1() -> Level {
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
            start_coordinate,
            velocity_magnitude: 500.0,
            facing: Facing::DownRight,
        }],
    }
}

fn get_level2() -> Level {
    let goal_coordinate = HexCoordinate { q: 7, r: 4 };
    let start_coordinate = HexCoordinate { q: 2, r: 1 };

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
        (HexCoordinate { q: 1, r: 1 }, TileType::SlowDown),
        (start_coordinate.clone(), TileType::MaintainSpeed),
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
            start_coordinate,
            velocity_magnitude: 100.0,
            facing: Facing::DownRight,
        }],
    }
}
