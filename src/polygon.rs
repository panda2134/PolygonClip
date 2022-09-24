use speedy2d::dimen::{Vec2, Vector2};
use crate::edge::Edge;
use crate::vec::{cross_product, inner_product};

/// edges are in counter-clockwise order
/// we have to use f64 in this function internally to avoid loss of precision
pub fn is_point_in_polygon (point: Vec2, polygon: &[Edge]) -> bool {
    let winding = polygon.iter().map(|edge| {
        let v1_32 = edge.from - point;
        let v2_32 = edge.to - point;
        let v1 = Vector2::new(v1_32.x as f64, v1_32.y as f64);
        let v2 = Vector2::new(v2_32.x as f64, v2_32.y as f64);
        (inner_product(&v1, &v2) / (
            (v1.x * v1.x + v1.y * v1.y).sqrt() * (v2.x * v2.x + v2.y * v2.y).sqrt()
        )).acos()
            * if cross_product(&v1, &v2) >= 0.0 { 1.0 } else { -1.0 }
    }).sum::<f64>();
    winding.abs() > 5e-4
}