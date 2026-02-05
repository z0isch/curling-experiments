use std::{collections::HashMap, fmt::Display, slice::Iter};

use bevy::prelude::*;

use crate::{hex_grid::HexCoordinate, tile::TileType};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CurrentLevel {
    #[default]
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
}

impl CurrentLevel {
    pub fn iterator() -> Iter<'static, CurrentLevel> {
        static LEVELS: [CurrentLevel; 7] = [
            CurrentLevel::Level0,
            CurrentLevel::Level1,
            CurrentLevel::Level2,
            CurrentLevel::Level3,
            CurrentLevel::Level4,
            CurrentLevel::Level5,
            CurrentLevel::Level6,
        ];
        LEVELS.iter()
    }
}

impl Display for CurrentLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentLevel::Level0 => write!(f, "Level 0"),
            CurrentLevel::Level1 => write!(f, "Level 1"),
            CurrentLevel::Level2 => write!(f, "Level 2"),
            CurrentLevel::Level3 => write!(f, "Level 3"),
            CurrentLevel::Level4 => write!(f, "Level 4"),
            CurrentLevel::Level5 => write!(f, "Level 5"),
            CurrentLevel::Level6 => write!(f, "Level 6"),
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
    pub countdown: Option<u32>,
    pub hex_radius: f32,
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
        CurrentLevel::Level0 => get_level0(),
        CurrentLevel::Level1 => get_level1(),
        CurrentLevel::Level2 => get_level2(),
        CurrentLevel::Level3 => get_level3(),
        CurrentLevel::Level4 => get_level4(),
        CurrentLevel::Level5 => get_level5(),
        CurrentLevel::Level6 => get_level6(),
    }
}

fn get_level0() -> Level {
    let grid = HashMap::from([(HexCoordinate { q: 0, r: 0 }, TileType::SlowDown)]);

    Level {
        hex_radius: 100.0,
        current_level: CurrentLevel::Level0,
        grid,
        goal_coordinate: HexCoordinate { q: 0, r: 0 },
        stone_configs: vec![],
        countdown: None,
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
        hex_radius: 60.0,
        current_level: CurrentLevel::Level1,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 200.0,
            facing: Facing::DownRight,
        }],
        countdown: Some(3),
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
        hex_radius: 60.0,
        current_level: CurrentLevel::Level2,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 190.0,
            facing: Facing::DownRight,
        }],
        countdown: Some(3),
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
        hex_radius: 60.0,
        current_level: CurrentLevel::Level3,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 200.0,
            facing: Facing::DownRight,
        }],
        countdown: Some(3),
    }
}

fn get_level4() -> Level {
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
        (
            HexCoordinate { q: 3, r: 2 },
            TileType::SpeedUp(Facing::UpRight),
        ),
        (HexCoordinate { q: 4, r: 1 }, TileType::MaintainSpeed),
        (HexCoordinate { q: 5, r: 1 }, TileType::MaintainSpeed),
        (HexCoordinate { q: 6, r: 0 }, TileType::MaintainSpeed),
        (goal_coordinate.clone(), TileType::Goal),
    ]);

    Level {
        hex_radius: 60.0,
        current_level: CurrentLevel::Level4,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 100.0,
            facing: Facing::DownRight,
        }],
        countdown: Some(3),
    }
}

fn get_level5() -> Level {
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
        hex_radius: 60.0,
        current_level: CurrentLevel::Level5,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 200.0,
            facing: Facing::DownRight,
        }],
        countdown: Some(3),
    }
}

fn get_level6() -> Level {
    let goal_coordinate = HexCoordinate { q: 3, r: 1 };
    let start_coordinate = HexCoordinate { q: 1, r: 2 };

    let grid = HashMap::from([
        (HexCoordinate { q: 0, r: 1 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 2 }, TileType::Wall),
        (HexCoordinate { q: 0, r: 3 }, TileType::Wall),
        //
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
        //
        //
        (HexCoordinate { q: 1, r: 1 }, TileType::Wall),
        (start_coordinate.clone(), TileType::SlowDown),
        (HexCoordinate { q: 1, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 1, r: 4 }, TileType::Wall),
        //
        (HexCoordinate { q: 2, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 2, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 2, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 2, r: 4 }, TileType::Wall),
        //
        (
            HexCoordinate { q: 3, r: 2 },
            TileType::SpeedUp(Facing::DownRight),
        ),
        (HexCoordinate { q: 3, r: 1 }, TileType::Goal),
        (HexCoordinate { q: 3, r: 3 }, TileType::Wall),
        (HexCoordinate { q: 3, r: 4 }, TileType::Wall),
        //
        (HexCoordinate { q: 4, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 4, r: 4 }, TileType::Wall),
        //
        (HexCoordinate { q: 5, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 2 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 3 }, TileType::SlowDown),
        (HexCoordinate { q: 5, r: 4 }, TileType::Wall),
        //
        (HexCoordinate { q: 6, r: 1 }, TileType::SlowDown),
        (HexCoordinate { q: 6, r: 2 }, TileType::SlowDown),
        (
            HexCoordinate { q: 6, r: 3 },
            TileType::SpeedUp(Facing::UpLeft),
        ),
        (HexCoordinate { q: 6, r: 4 }, TileType::Wall),
    ]);

    Level {
        hex_radius: 60.0,
        current_level: CurrentLevel::Level6,
        grid,
        goal_coordinate,
        stone_configs: vec![StoneConfig {
            start_coordinate,
            velocity_magnitude: 250.0,
            facing: Facing::DownRight,
        }],
        countdown: Some(3),
    }
}
