use bevy::prelude::*;

use crate::{
    level::Level,
    screens::Screen,
    tile::{
        CanBeDragged, IsGoal, ScratchOffMaterial, TileAssets, TileType, on_pointer_out,
        on_pointer_over, on_tile_drag_end, on_tile_drag_enter, on_tile_drag_leave,
        on_tile_dragging, tile, tile_can_be_dragged,
    },
};

/// Component for the hex grid entity.
/// Tiles are spawned as children of this entity.
#[derive(Component, Clone)]
pub struct HexGrid {
    pub hex_radius: f32,
    pub horiz_spacing: f32,
    pub vert_spacing: f32,
    pub cols: (i32, i32),
    pub rows: (i32, i32),
    pub offset_x: f32,
    pub offset_y: f32,
    pub level: Level,
}

impl HexGrid {
    pub fn new(level: &Level) -> Self {
        let hex_radius = level.hex_radius;
        let cols = (
            level.grid.keys().map(|coord| coord.q).min().unwrap_or(0),
            level.grid.keys().map(|coord| coord.q).max().unwrap_or(0) + 1,
        );
        let rows = (
            level.grid.keys().map(|coord| coord.r).min().unwrap_or(0),
            level.grid.keys().map(|coord| coord.r).max().unwrap_or(0) + 1,
        );
        let horiz_spacing = hex_radius * 1.5;
        let vert_spacing = hex_radius * 3.0_f32.sqrt();

        // Center the grid horizontally based on the center of the q coordinate range
        let center_q = (cols.0 as f32 + (cols.1 - 1) as f32) / 2.0;
        let offset_x = -center_q * horiz_spacing;

        // Center the grid vertically (accounting for row inversion in hex_to_world)
        let num_rows = rows.1 - rows.0;
        let offset_y = -((num_rows - 1) as f32 / 2.0) * vert_spacing;

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
    // Use != 0 instead of == 1 to handle negative odd numbers correctly
    let y_offset = if hex_coord.q % 2 != 0 {
        hex_grid.vert_spacing / 2.0
    } else {
        0.0
    };
    let y = hex_grid.offset_y
        + (hex_grid.rows.1 - 1 - hex_coord.r) as f32 * hex_grid.vert_spacing
        + y_offset;

    Vec2::new(x, y)
}

pub fn spawn_hex_grid(
    commands: &mut Commands,
    grid: &HexGrid,
    tile_assets: &TileAssets,
    level: &Level,
    scratch_materials: &mut Assets<ScratchOffMaterial>,
) -> Entity {
    let mut tile_entities = Vec::new();

    for q in grid.cols.0..grid.cols.1 {
        for r in grid.rows.0..grid.rows.1 {
            let world_pos = hex_to_world(&HexCoordinate { q, r }, grid);
            if let Some(tile_type) = grid.level.grid.get(&HexCoordinate { q, r }) {
                let tile_id = commands
                    .spawn((tile(
                        tile_type,
                        world_pos,
                        q,
                        r,
                        level.min_sweep_distance,
                        tile_assets,
                        scratch_materials,
                    ),))
                    .observe(on_pointer_over)
                    .observe(on_pointer_out)
                    .observe(on_tile_dragging)
                    .observe(on_tile_drag_enter)
                    .observe(on_tile_drag_end)
                    .observe(on_tile_drag_leave)
                    .id();
                if tile_can_be_dragged(tile_type) {
                    commands.entity(tile_id).insert(CanBeDragged);
                }
                if tile_type == &TileType::Goal {
                    commands.entity(tile_id).insert(IsGoal);
                }
                tile_entities.push(tile_id);
            }
        }
    }

    commands
        .spawn((
            DespawnOnExit(Screen::Gameplay),
            Visibility::Visible,
            Transform::from_xyz(0., 0., 0.),
            grid.clone(),
        ))
        .add_children(&tile_entities)
        .id()
}
