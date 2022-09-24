use speedy2d::dimen::{Vec2, Vector2};

use crate::edge::Edge;
use crate::vec::{cross_product, inner_product};

/// edges are in counter-clockwise order
pub fn is_point_in_polygon (point: Vec2, polygon: &[Edge]) -> bool {
    let winding = polygon.iter().map(|edge| {
        let v1_32 = edge.from - point;
        let v2_32 = edge.to - point;
        get_directed_angle(v1_32, v2_32)
    }).sum::<f64>();
    winding.abs() > 5e-4
}

/// calculate the directed angle between 2 vectors
/// clockwise = + angle
/// we have to use f64 in this function internally to avoid loss of precision
fn get_directed_angle(v1_32: Vector2<f32>, v2_32: Vector2<f32>) -> f64 {
    let v1 = Vector2::new(v1_32.x as f64, v1_32.y as f64);
    let v2 = Vector2::new(v2_32.x as f64, v2_32.y as f64);
    (inner_product(&v1, &v2) / (
        (v1.x * v1.x + v1.y * v1.y).sqrt() * (v2.x * v2.x + v2.y * v2.y).sqrt()
    )).acos()
        * if cross_product(&v1, &v2) >= 0.0 { 1.0 } else { -1.0 }
}

/// supposed coordinate system: x to the right, y downwards
/// thus, positive cross product indicates going clockwise
pub fn is_polygon_clockwise (polygon: &[Edge]) -> bool {
    let total_angle = polygon.iter()
        .zip(polygon.iter().skip(1).chain(polygon.iter().take(1)))
        .map(|(e1, e2)| get_directed_angle(e1.get_vector(), e2.get_vector()))
        .sum::<f64>();

    total_angle > 0.0
}