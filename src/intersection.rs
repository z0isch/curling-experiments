use bevy::math::{
    Vec2,
    bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
};

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

pub fn ratio_circle_area_inside_hexagon(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
    samples: u32,
) -> f32 {
    let area = circle_area_inside_hexagon(
        circle_center,
        circle_radius,
        hex_center,
        hex_radius,
        samples,
    );
    let circle_area = std::f32::consts::PI * circle_radius * circle_radius;
    area / circle_area
}

pub fn circle_area_inside_hexagon(
    circle_center: Vec2,
    circle_radius: f32,
    hex_center: Vec2,
    hex_radius: f32,
    samples: u32,
) -> f32 {
    if !aabb_intersects(circle_center, circle_radius, hex_center, hex_radius) {
        return 0.0;
    }

    let circle_points = approximate_circle_points(circle_radius, circle_center, samples);
    let hex_points = hexagon_points(hex_radius, hex_center);
    let clipped_points = clip_polygon_sutherland_hodgman(&circle_points, &hex_points);
    polygon_area(&clipped_points)
}

/// Calculates the area of a polygon using the Shoelace formula.
///
/// The polygon vertices should be in order (either clockwise or counter-clockwise).
/// Returns the absolute area value.
fn polygon_area(points: &[Vec2]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut sum = 0.0;
    let n = points.len();

    for i in 0..n {
        let current = points[i];
        let next = points[(i + 1) % n];
        sum += current.x * next.y - next.x * current.y;
    }

    (sum / 2.0).abs()
}
/// Returns the vertices of a flat-top hexagon in counter-clockwise order.
fn hexagon_points(radius: f32, center: Vec2) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(6);

    for i in 0..6 {
        // Start at 0° (right vertex) and go counter-clockwise in 60° increments
        let angle = std::f32::consts::PI / 3.0 * (i as f32);
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        points.push(Vec2::new(x, y));
    }

    points
}

/// Traces around a circle and returns points on the circumference in counter-clockwise order.
///
/// Points are evenly distributed starting from the rightmost point (angle 0)
/// and proceeding clockwise.
fn approximate_circle_points(radius: f32, center: Vec2, samples: u32) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(samples as usize);

    for i in 0..samples {
        // Counter-clockwise means positive angle direction in standard 2D coords (y-up)
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (samples as f32);
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        points.push(Vec2::new(x, y));
    }

    points
}

/// Clips a polygon against a convex clip polygon using the Sutherland-Hodgman algorithm.
///
/// Returns the vertices of the clipped polygon, or an empty vector if there's no intersection.
/// The clip polygon must be convex and its vertices should be in counter-clockwise order.
fn clip_polygon_sutherland_hodgman(polygon: &[Vec2], clip_polygon: &[Vec2]) -> Vec<Vec2> {
    if polygon.is_empty() || clip_polygon.len() < 3 {
        return Vec::new();
    }

    let mut output = polygon.to_vec();

    // Process each edge of the clip polygon
    for i in 0..clip_polygon.len() {
        if output.is_empty() {
            return Vec::new();
        }

        let edge_start = clip_polygon[i];
        let edge_end = clip_polygon[(i + 1) % clip_polygon.len()];

        let input = output;
        output = Vec::new();

        for j in 0..input.len() {
            let current = input[j];
            let previous = input[(j + input.len() - 1) % input.len()];

            let current_inside = is_inside_edge(current, edge_start, edge_end);
            let previous_inside = is_inside_edge(previous, edge_start, edge_end);

            if current_inside {
                if !previous_inside {
                    // Entering the clip region - add intersection point
                    if let Some(intersection) =
                        line_segment_intersection(previous, current, edge_start, edge_end)
                    {
                        output.push(intersection);
                    }
                }
                // Current vertex is inside - add it
                output.push(current);
            } else if previous_inside {
                // Leaving the clip region - add intersection point
                if let Some(intersection) =
                    line_segment_intersection(previous, current, edge_start, edge_end)
                {
                    output.push(intersection);
                }
            }
        }
    }

    output
}

/// Determines if a point is on the "inside" (left side) of a directed edge.
/// For a counter-clockwise polygon, inside means to the left of the edge direction.
fn is_inside_edge(point: Vec2, edge_start: Vec2, edge_end: Vec2) -> bool {
    // Cross product of edge vector and point vector
    // Positive means point is to the left (inside for CCW polygon)
    let edge = edge_end - edge_start;
    let to_point = point - edge_start;
    edge.x * to_point.y - edge.y * to_point.x >= 0.0
}

/// Computes the intersection point of two line segments.
/// Returns None if the lines are parallel.
fn line_segment_intersection(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    let d1 = p2 - p1;
    let d2 = p4 - p3;

    let cross = d1.x * d2.y - d1.y * d2.x;

    // Lines are parallel
    if cross.abs() < 1e-10 {
        return None;
    }

    let d3 = p3 - p1;
    let t = (d3.x * d2.y - d3.y * d2.x) / cross;

    Some(p1 + d1 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Vec2::new(46.0, 0.0),
            35.0
        ));
    }

    #[test]
    fn test_clip_polygon_square_overlap() {
        // Two overlapping squares (CCW order)
        let subject = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        let clip = vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(3.0, 1.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(1.0, 3.0),
        ];

        let result = clip_polygon_sutherland_hodgman(&subject, &clip);

        // Should produce a 1x1 square at (1,1) to (2,2)
        assert_eq!(result.len(), 4);

        // Verify the clipped polygon contains the expected intersection region
        for point in &result {
            assert!(point.x >= 1.0 - 1e-5 && point.x <= 2.0 + 1e-5);
            assert!(point.y >= 1.0 - 1e-5 && point.y <= 2.0 + 1e-5);
        }
    }

    #[test]
    fn test_clip_polygon_no_overlap() {
        // Two non-overlapping squares
        let subject = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        let clip = vec![
            Vec2::new(5.0, 5.0),
            Vec2::new(6.0, 5.0),
            Vec2::new(6.0, 6.0),
            Vec2::new(5.0, 6.0),
        ];

        let result = clip_polygon_sutherland_hodgman(&subject, &clip);
        assert!(result.is_empty());
    }

    #[test]
    fn test_clip_polygon_fully_inside() {
        // Small square fully inside larger square
        let subject = vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(1.0, 2.0),
        ];
        let clip = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(0.0, 5.0),
        ];

        let result = clip_polygon_sutherland_hodgman(&subject, &clip);

        // Should return the original subject polygon
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_clip_polygon_empty_input() {
        let empty: Vec<Vec2> = vec![];
        let clip = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
        ];

        let result = clip_polygon_sutherland_hodgman(&empty, &clip);
        assert!(result.is_empty());
    }

    #[test]
    fn test_circle_area_inside_hexagon_fully_inside() {
        // Small circle at center of large hexagon - should return full circle area
        let circle_radius = 5.0;
        let hex_radius = 50.0;
        let samples = 64;

        let area =
            circle_area_inside_hexagon(Vec2::ZERO, circle_radius, Vec2::ZERO, hex_radius, samples);

        let expected_circle_area = std::f32::consts::PI * circle_radius * circle_radius;
        // With 64 samples, the polygon approximation should be very close
        assert!(
            (area - expected_circle_area).abs() < expected_circle_area * 0.05,
            "Expected area ~{}, got {}",
            expected_circle_area,
            area
        );
    }

    #[test]
    fn test_circle_area_inside_hexagon_fully_outside() {
        // Circle far from hexagon - should return 0
        let area = circle_area_inside_hexagon(Vec2::new(100.0, 100.0), 10.0, Vec2::ZERO, 35.0, 64);

        assert_eq!(area, 0.0);
    }

    #[test]
    fn test_circle_area_inside_hexagon_partial_overlap() {
        // Circle partially overlapping hexagon
        let circle_radius = 20.0;
        let hex_radius = 35.0;

        // Place circle at the edge of the hexagon
        let area = circle_area_inside_hexagon(
            Vec2::new(hex_radius, 0.0),
            circle_radius,
            Vec2::ZERO,
            hex_radius,
            64,
        );

        let full_circle_area = std::f32::consts::PI * circle_radius * circle_radius;
        // Should be between 0 and full circle area
        assert!(area > 0.0, "Expected positive area, got {}", area);
        assert!(
            area < full_circle_area,
            "Expected area < {}, got {}",
            full_circle_area,
            area
        );
    }

    #[test]
    fn test_circle_area_inside_hexagon_more_samples_more_accurate() {
        // More samples should give a more accurate circle approximation
        let circle_radius = 10.0;
        let hex_radius = 50.0;
        let expected_area = std::f32::consts::PI * circle_radius * circle_radius;

        let area_low =
            circle_area_inside_hexagon(Vec2::ZERO, circle_radius, Vec2::ZERO, hex_radius, 8);
        let area_high =
            circle_area_inside_hexagon(Vec2::ZERO, circle_radius, Vec2::ZERO, hex_radius, 128);

        // Higher sample count should be closer to the true circle area
        let error_low = (area_low - expected_area).abs();
        let error_high = (area_high - expected_area).abs();

        assert!(
            error_high < error_low,
            "Higher samples should be more accurate: error_low={}, error_high={}",
            error_low,
            error_high
        );
    }
}
