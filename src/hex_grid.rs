use bevy::prelude::*;

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
}

impl HexGrid {
    pub fn new(hex_radius: f32, cols: i32, rows: i32) -> Self {
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
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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

/// Converts world position to hex grid coordinates for flat-top hexagons
pub fn world_to_hex(world_pos: Vec2, hex_grid: &HexGrid) -> Option<HexCoordinate> {
    // Translate position relative to grid origin
    let rel_x = world_pos.x - hex_grid.offset_x;
    let rel_y = world_pos.y - hex_grid.offset_y;

    // Estimate column (accounting for horizontal spacing)
    let q_estimate = (rel_x / hex_grid.horiz_spacing).round() as i32;

    // Check bounds
    if q_estimate < 0 || q_estimate >= hex_grid.cols {
        return None;
    }

    // Account for vertical offset on odd columns
    let y_offset = if q_estimate % 2 == 1 {
        hex_grid.vert_spacing / 2.0
    } else {
        0.0
    };

    // Estimate row (r=0 at top, inverted from y coordinate)
    let visual_r = ((rel_y - y_offset) / hex_grid.vert_spacing).round() as i32;
    let r_estimate = (hex_grid.rows - 1) - visual_r;

    // Check bounds
    if r_estimate < 0 || r_estimate >= hex_grid.rows {
        return None;
    }

    // Calculate the center of this hex cell (using inverted r for y position)
    let hex_center_x = hex_grid.offset_x + q_estimate as f32 * hex_grid.horiz_spacing;
    let hex_center_y = hex_grid.offset_y
        + (hex_grid.rows - 1 - r_estimate) as f32 * hex_grid.vert_spacing
        + y_offset;

    // Check if point is actually within the hexagon (using distance check)
    // For flat-top hexagons, the inner radius (apothem) = radius * sqrt(3)/2
    let dx = (world_pos.x - hex_center_x).abs();
    let dy = (world_pos.y - hex_center_y).abs();

    // Simple bounding check using the hexagon's geometry
    let inner_radius = hex_grid.hex_radius * 3.0_f32.sqrt() / 2.0;

    // For a flat-top hexagon, check if point is inside
    // Using the hex boundary equations
    if dx > hex_grid.hex_radius || dy > inner_radius {
        return None;
    }

    // More precise check for the angled edges
    // For flat-top hex: the slanted edges have slope related to the hex geometry
    if dx * inner_radius + dy * hex_grid.hex_radius / 2.0 > hex_grid.hex_radius * inner_radius {
        return None;
    }

    Some(HexCoordinate {
        q: q_estimate,
        r: r_estimate,
    })
}

/// Creates a bundle for spawning a hex grid entity
pub fn hex_grid(hex_radius: f32, cols: i32, rows: i32) -> impl Bundle {
    (
        Visibility::Visible,
        Transform::from_xyz(0., 0., 0.),
        HexGrid::new(hex_radius, cols, rows),
    )
}
