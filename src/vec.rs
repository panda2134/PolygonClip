use speedy2d::dimen::{Vec2, Vector2};

pub fn inner_product<T: num_traits::float::Float>(lhs: &Vector2<T>, rhs: &Vector2<T>) -> T {
    lhs.x * rhs.x + lhs.y * rhs.y
}

pub fn cross_product<T: num_traits::float::Float>(lhs: &Vector2<T>, rhs: &Vector2<T>) -> T {
    lhs.x * rhs.y - rhs.x * lhs.y
}

/// suppose the line goes through origin.
pub fn dist_to_line(line: &Vec2, point: &Vec2) -> f32 {
    cross_product(line, point).abs() / line.magnitude()
}