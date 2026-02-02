//! Intersection area calculations between shapes.
//!
//! Uses AABB collision detection as a fast first pass, then Monte Carlo
//! sampling for accurate intersection area approximation.

use bevy::math::{
    Vec2,
    bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
};
use rand::Rng;

/// Calculates the approximate intersection area between a circle (stone) and a flat-top hexagon (tile).
///
/// Returns `0.0` if the shapes don't intersect (based on AABB check).
pub fn circle_hexagon_intersection_area(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
    samples: u32,
) -> f32 {
    // Fast AABB collision check first
    if !aabb_intersects(circle_center, circle_radius, hex_center, hex_radius) {
        return 0.0;
    }

    // Monte Carlo sampling for intersection area
    sample_intersection_area(
        circle_center,
        circle_radius,
        hex_center,
        hex_radius,
        samples,
    )
}

/// Fast AABB intersection check between a circle and a flat-top hexagon.
pub fn aabb_intersects(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
) -> bool {
    let circle_aabb = BoundingCircle::new(circle_center, circle_radius).aabb_2d();

    // For a flat-top hexagon:
    // - Width (horizontal span) = 2 * radius
    // - Height (vertical span) = sqrt(3) * radius
    let hex_half_extents = Vec2::new(hex_radius, hex_radius * 3.0_f32.sqrt() / 2.0);
    let hex_aabb = Aabb2d::new(hex_center, hex_half_extents);

    circle_aabb.intersects(&hex_aabb)
}

/// Monte Carlo sampling to approximate the intersection area.
///
/// Uses rejection sampling to generate uniform points within the circle,
/// then counts how many fall inside the hexagon.
fn sample_intersection_area(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
    samples: u32,
) -> f32 {
    let mut rng = rand::rng();
    let mut hits = 0u32;
    let mut valid_samples = 0u32;

    let circle = BoundingCircle::new(circle_center, circle_radius);

    // Use rejection sampling to get uniform samples within the circle
    while valid_samples < samples {
        let x = circle_center.x + rng.random_range(-circle_radius..circle_radius);
        let y = circle_center.y + rng.random_range(-circle_radius..circle_radius);
        let point = Vec2::new(x, y);

        // Only count points that are inside the circle
        if circle.contains(&BoundingCircle::new(point, 0.0)) {
            valid_samples += 1;

            // Check if point is also inside the hexagon
            if point_in_flat_top_hexagon(point, hex_center, hex_radius) {
                hits += 1;
            }
        }
    }

    // Approximate area = (hits / valid_samples) * circle_area
    let circle_area = std::f32::consts::PI * circle_radius * circle_radius;
    (hits as f32 / valid_samples as f32) * circle_area
}

/// Check if a point is inside a flat-top hexagon.
///
/// A flat-top hexagon has vertices at angles 30°, 90°, 150°, 210°, 270°, 330°
/// (i.e., rotated 30° from a pointy-top hexagon).
pub fn point_in_flat_top_hexagon(point: Vec2, center: Vec2, radius: f32) -> bool {
    signed_distance_to_flat_top_hexagon(point, center, radius) <= 0.0
}

/// Returns the signed distance from a point to a flat-top hexagon.
/// Negative values mean the point is inside, positive means outside.
fn signed_distance_to_flat_top_hexagon(point: Vec2, center: Vec2, radius: f32) -> f32 {
    let rel = point - center;
    let dx = rel.x.abs();
    let dy = rel.y.abs();

    // Inner radius (apothem) for flat-top hex = radius * sqrt(3) / 2
    let inner_radius = radius * 3.0_f32.sqrt() / 2.0;

    // For a flat-top hex, we check three half-planes:
    // 1. Right edge: x <= radius
    // 2. Top/bottom edges: y <= inner_radius
    // 3. Diagonal edges: dx * inner_radius + dy * (radius / 2) <= radius * inner_radius

    // Distance to each constraint (positive = outside)
    let dist_right = dx - radius;
    let dist_top = dy - inner_radius;

    // For the diagonal edge, normalize the constraint equation
    // The edge normal has components (inner_radius, radius/2), normalize it
    let diag_normal_len = (inner_radius * inner_radius + (radius / 2.0) * (radius / 2.0)).sqrt();
    let dist_diagonal =
        (dx * inner_radius + dy * radius / 2.0 - radius * inner_radius) / diag_normal_len;

    // Return the maximum distance (most constraining)
    dist_right.max(dist_top).max(dist_diagonal)
}

/// Check if a circle intersects with a flat-top hexagon.
/// Returns true if any part of the circle overlaps with the hexagon.
pub fn circle_intersects_flat_top_hexagon(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
) -> bool {
    // The circle intersects the hex if the signed distance from
    // the circle center to the hex is less than the circle radius
    signed_distance_to_flat_top_hexagon(circle_center, hex_center, hex_radius) < circle_radius
}

/// Returns the percentage of the circle that overlaps with the hexagon (0.0 to 1.0).
pub fn circle_hexagon_overlap_ratio(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
    samples: u32,
) -> f32 {
    let intersection_area = circle_hexagon_intersection_area(
        circle_center,
        circle_radius,
        hex_center,
        hex_radius,
        samples,
    );
    let circle_area = std::f32::consts::PI * circle_radius * circle_radius;

    if circle_area > 0.0 {
        intersection_area / circle_area
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_intersection_far_apart() {
        let area = circle_hexagon_intersection_area(
            Vec2::new(0.0, 0.0),
            10.0,
            Vec2::new(100.0, 100.0),
            35.0,
            100,
        );
        assert_eq!(area, 0.0);
    }

    #[test]
    fn test_circle_fully_inside_hex() {
        // Small circle at center of large hex should be ~100% inside
        let ratio =
            circle_hexagon_overlap_ratio(Vec2::new(0.0, 0.0), 5.0, Vec2::new(0.0, 0.0), 50.0, 1000);
        assert!(ratio > 0.95, "Expected ratio > 0.95, got {}", ratio);
    }

    #[test]
    fn test_point_in_flat_top_hexagon_center() {
        assert!(point_in_flat_top_hexagon(Vec2::ZERO, Vec2::ZERO, 35.0));
    }

    #[test]
    fn test_point_in_flat_top_hexagon_outside() {
        // Point clearly outside
        assert!(!point_in_flat_top_hexagon(
            Vec2::new(100.0, 100.0),
            Vec2::ZERO,
            35.0
        ));
    }

    #[test]
    fn test_aabb_intersects_overlapping() {
        assert!(aabb_intersects(
            Vec2::new(0.0, 0.0),
            10.0,
            Vec2::new(15.0, 0.0),
            35.0
        ));
    }

    #[test]
    fn test_aabb_intersects_not_overlapping() {
        assert!(!aabb_intersects(
            Vec2::new(0.0, 0.0),
            10.0,
            Vec2::new(45.0, 0.0),
            35.0
        ));
    }
}
