use speedy2d::dimen::Vec2;
use crate::vec::{cross_product, dist_to_line, inner_product};

#[derive(Copy, Clone, Debug)]
pub struct Edge {
    pub from: Vec2,
    pub to: Vec2
}

impl Edge {
    pub fn get_vector(&self) -> Vec2 {
        self.to - self.from
    }
    pub fn has_point (&self, point: &Vec2) -> bool {
        let from_point_vec = point - self.from;
        let self_vec = self.get_vector();
        if from_point_vec.magnitude() <= f32::EPSILON { return true }
        let same_line = (1.0 - inner_product(&self_vec.normalize().unwrap(),
                                             &from_point_vec.normalize().unwrap())).abs()
            <= f32::EPSILON;
        let on_edge = from_point_vec.magnitude_squared() <= self_vec.magnitude_squared();
        same_line && on_edge
    }
    pub fn intersect_with (&self, other: &Self) -> Option<Vec2> {
        let self_vec = self.get_vector();
        let other_vec = other.get_vector();
        if cross_product(&self_vec, &other_vec).abs() <= f32::EPSILON {
            // parallel
            // (x-x0) * v = (y-y0) * u
            // v * x + (-u) * y + (u * y0 - v * x0)
            let dist = dist_to_line(&self.get_vector(), &(other.from - self.from));
            if dist > f32::EPSILON { None }
            else if self.has_point(&other.from) { Some(other.from) }
            else if self.has_point(&other.to) { Some(other.to) }
            else { None }
        } else {
            // i = p0 + k * v0
            // i = p1 + k' * v1
            // p0 + k * v0 = p1 + k' * v1
            // p0.x + k * v0.x = p1.x + k' * v1.x;  p0.y + k * v0.y = p1.y + k' * v1.y
            // k * v0.x - k' * v1.x = p1.x - p0.x
            // k * v0.y - k' * v1.y = p1.y - p0.y
            // (p1.x - p0.x - k * v0.x) / v1.x = (p1.y - p0.y - k * v0.y) / v1.y
            // v1.y * p1.x - v1.y * p0.x - v1.y * k * v0.x = p1.y * v1.x - p0.y * v1.x - k * v0.y * v1.x
            // v1.y * p1.x - v1.y * p0.x + p0.y * v1.x - p1.y * v1.x = k * (v0.x * v1.y - v0.y * v1.x)
            let b = other_vec.y * (other.from.x - self.from.x) + other_vec.x * (self.from.y - other.from.y);
            let k = b / cross_product(&self_vec, &other_vec); // cross product non-zero now
            let intersection = self.from + self_vec * k;
            if !(0.0..=1.0).contains(&k) || !other.has_point(&intersection) { None }
                else { Some(intersection) }
        }
    }
}

#[cfg(test)]
mod tests {
    use speedy2d::dimen::Vec2;
    use crate::edge::Edge;

    #[test]
    fn test_has_point() {
        let edge = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 2.0 } };
        assert!(edge.has_point(&Vec2 { x: 2.0, y: 1.5 }), "fail on basic case");
        assert!(edge.has_point(&Vec2 { x: 1.0, y: 1.0 }), "fail on edge start");
        assert!(edge.has_point(&Vec2 { x: 3.0, y: 2.0 }), "fail on edge end");
        assert!(! edge.has_point(&Vec2 { x: 2.0, y: 1.6 }), "fail on non-intersecting");
        assert!(! edge.has_point(&Vec2 { x: 2.0, y: 1.0 }), "fail on non-intersecting");
    }

    #[test]
    fn test_intersection() {
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 3.0, y: 1.0 }, to: Vec2 { x: 1.0, y: 3.0 } };
            assert_eq!(lhs.intersect_with(&rhs), Some(Vec2 {x: 2.0, y: 2.0}));
            assert_eq!(rhs.intersect_with(&lhs), Some(Vec2 {x: 2.0, y: 2.0}));
        }
        {

            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 3.0, y: 1.0 }, to: Vec2 { x: 5.0, y: -3.0 } };
            assert_eq!(lhs.intersect_with(&rhs), None);
            assert_eq!(rhs.intersect_with(&lhs), None);
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 3.0, y: 1.0 }, to: Vec2 { x: 5.0, y: 3.0 } };
            assert_eq!(lhs.intersect_with(&rhs), None);
            assert_eq!(rhs.intersect_with(&lhs), None);
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 4.0, y: 4.0 }, to: Vec2 { x: 5.0, y: 5.0 } };
            assert_eq!(lhs.intersect_with(&rhs), None);
            assert_eq!(rhs.intersect_with(&lhs), None);
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 3.0, y: 3.0 }, to: Vec2 { x: 4.0, y: 4.0 } };
            assert_eq!(lhs.intersect_with(&rhs), Some(Vec2 { x: 3.0, y: 3.0 }));
            assert_eq!(rhs.intersect_with(&lhs), Some(Vec2 { x: 3.0, y: 3.0 }));
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 3.0, y: 3.0 }, to: Vec2 { x: 3.0, y: 4.0 } };
            assert_eq!(lhs.intersect_with(&rhs), Some(Vec2 { x: 3.0, y: 3.0 }));
            assert_eq!(rhs.intersect_with(&lhs), Some(Vec2 { x: 3.0, y: 3.0 }));
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 {x: 4.0, y: 4.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            assert_eq!(lhs.intersect_with(&rhs), Some(Vec2 { x: 3.0, y: 3.0 }));
            assert_eq!(rhs.intersect_with(&lhs), Some(Vec2 { x: 3.0, y: 3.0 }));
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 { x: 4.0, y: 4.0 }, to: Vec2 { x: 5.0, y: 4.9 } };
            assert_eq!(lhs.intersect_with(&rhs), None);
            assert_eq!(rhs.intersect_with(&lhs), None);
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 { x: 2.0, y: 2.0 }, to: Vec2 { x: 5.0, y: 4.9 } };
            assert_eq!(lhs.intersect_with(&rhs), Some(Vec2 { x: 2.0, y: 2.0 }));
            assert_eq!(rhs.intersect_with(&lhs), Some(Vec2 { x: 2.0, y: 2.0 }));
        }
        {
            let lhs = Edge { from: Vec2 { x: 1.0, y: 1.0 }, to: Vec2 { x: 3.0, y: 3.0 } };
            let rhs = Edge { from: Vec2 { x: 4.0, y: 2.0 }, to: Vec2 { x: 2.0, y: 2.0 } };
            assert_eq!(lhs.intersect_with(&rhs), Some(Vec2 { x: 2.0, y: 2.0 }));
            assert_eq!(rhs.intersect_with(&lhs), Some(Vec2 { x: 2.0, y: 2.0 }));
        }
    }
}