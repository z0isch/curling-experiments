use std::{collections::HashMap, fmt::Display, slice::Iter};

use bevy::prelude::*;

use crate::{hex_grid::HexCoordinate, tile::TileType};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CurrentLevel {
    #[default]
    Level1,
    Level2,
    Level3,
    Level4,
}

impl CurrentLevel {
    pub fn iterator() -> Iter<'static, CurrentLevel> {
        static LEVELS: [CurrentLevel; 4] = [
            CurrentLevel::Level1,
            CurrentLevel::Level2,
            CurrentLevel::Level3,
            CurrentLevel::Level4,
        ];
        LEVELS.iter()
    }
}

impl Display for CurrentLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentLevel::Level1 => write!(f, "Level 1"),
            CurrentLevel::Level2 => write!(f, "Level 2"),
            CurrentLevel::Level3 => write!(f, "Level 3"),
            CurrentLevel::Level4 => write!(f, "Level 4"),
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

    pub fn to_vector(&self) -> Vec2 {
        Vec2::from_angle(self.to_angle())
    }

    pub fn to_angle(&self) -> f32 {
        use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_6};
        match self {
            Facing::Up => FRAC_PI_2,                    // 90° - straight up
            Facing::UpRight => FRAC_PI_6,               // 30° - up and right
            Facing::DownRight => -FRAC_PI_6,            // -30° - down and right
            Facing::Down => -FRAC_PI_2,                 // -90° - straight down
            Facing::DownLeft => -FRAC_PI_2 - FRAC_PI_3, // -120° - down and left
            Facing::UpLeft => FRAC_PI_2 + FRAC_PI_3,    // 120° - up and left
        }
    }
}

impl Display for Facing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Resource, Clone, PartialEq, Debug)]
pub struct Level {
    pub current_level: CurrentLevel,
    pub grid: HashMap<HexCoordinate, TileType>,
    pub goal_coordinate: HexCoordinate,
    pub stone_configs: Vec<StoneConfig>,
    pub countdown: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StoneConfig {
    pub velocity_magnitude: f32,
    pub start_coordinate: HexCoordinate,
    pub facing: Facing,
}

pub fn get_initial_stone_velocity(facing: &Facing, stone_velocity_magnitude: &f32) -> Vec2 {
    Facing::to_vector(facing) * *stone_velocity_magnitude
}

pub fn get_level(current_level: CurrentLevel) -> Level {
    match current_level {
        CurrentLevel::Level1 => get_level1(),
        CurrentLevel::Level2 => get_level2(),
        CurrentLevel::Level3 => get_level3(),
        CurrentLevel::Level4 => get_level4(),
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
        current_level: CurrentLevel::Level1,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 200.0,
            facing: Facing::DownRight,
        }],
        countdown: 3,
    }
}

fn get_level2() -> Level {
    let goal_coordinate = HexCoordinate { q: 7, r: 0 };
    let start_coordinate = HexCoordinate { q: 1, r: 1 };

    let grid = HashMap::from([
        (HexCoordinate { q: 0, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 6, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 8, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 8, r: -1 }, TileType::Wall),
        (HexCoordinate { q: 7, r: -1 }, TileType::Wall),
        (HexCoordinate { q: 6, r: -1 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 0 }, TileType::Wall),
        //
        (HexCoordinate { q: 8, r: 0 }, TileType::Wall),
        //
        (start_coordinate.clone(), TileType::MaintainSpeed),
        (HexCoordinate { q: 2, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 3, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 6, r: 0 }, TileType::SlowDown),
        (goal_coordinate.clone(), TileType::Goal),
    ]);

    Level {
        current_level: CurrentLevel::Level2,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 190.0,
            facing: Facing::DownRight,
        }],
        countdown: 3,
    }
}

fn get_level3() -> Level {
    let goal_coordinate = HexCoordinate { q: 6, r: 1 };
    let start_coordinate = HexCoordinate { q: 1, r: 1 };

    let grid = HashMap::from([
        (HexCoordinate { q: 0, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 6, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 6, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 0 }, TileType::Wall),
        //
        (start_coordinate.clone(), TileType::MaintainSpeed),
        (HexCoordinate { q: 2, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 3, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 1 }, TileType::SlowDown),
        (goal_coordinate.clone(), TileType::Goal),
    ]);

    Level {
        current_level: CurrentLevel::Level3,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 200.0,
            facing: Facing::DownRight,
        }],
        countdown: 3,
    }
}

fn get_level4() -> Level {
    let goal_coordinate = HexCoordinate { q: 6, r: 4 };
    let start_coordinate = HexCoordinate { q: 1, r: 1 };

    let grid = HashMap::from([
        (HexCoordinate { q: 0, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 4 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 5 }, TileType::Wall),
        //
        (HexCoordinate { q: 1, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 6, r: 5 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 5 }, TileType::Wall),
        //
        (HexCoordinate { q: 7, r: 4 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 7, r: 0 }, TileType::Wall),
        //
        (HexCoordinate { q: 6, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 5, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 2, r: 0 }, TileType::Wall),
        (HexCoordinate { q: 1, r: 0 }, TileType::Wall),
        //
        //
        (start_coordinate.clone(), TileType::MaintainSpeed),
        (HexCoordinate { q: 1, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 1, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 1, r: 4 }, TileType::SlowDown),
        //
        (HexCoordinate { q: 2, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 2, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 2, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 2, r: 4 }, TileType::SlowDown),
        //
        (HexCoordinate { q: 3, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 3, r: 3 }, TileType::SlowDown),
        (
            HexCoordinate { q: 3, r: 4 },
            TileType::SpeedUp(Facing::UpRight),
        ),
        //
        (HexCoordinate { q: 4, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 4, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 4 }, TileType::SlowDown),
        //
        (HexCoordinate { q: 5, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 4 }, TileType::Wall),
        //
        (
            HexCoordinate { q: 6, r: 1 },
            TileType::SpeedUp(Facing::Down),
        ),
        (HexCoordinate { q: 6, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 6, r: 3 }, TileType::SlowDown),
        (goal_coordinate.clone(), TileType::Goal),
    ]);

    Level {
        current_level: CurrentLevel::Level4,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 200.0,
            facing: Facing::DownRight,
        }],
        countdown: 3,
    }
}
