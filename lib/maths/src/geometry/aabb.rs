use crate::linear::Vec2f;

use super::{shape::Shape, Segment};

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec2f,
    pub max: Vec2f,
}

impl AABB {
    pub fn new(min: Vec2f, max: Vec2f) -> Self {
        Self { min, max }
    }

    pub fn from_dimensions(centre: Vec2f, dimensions: Vec2f) -> Self {
        let half_dimensions = dimensions * 0.5;
        return Self::new(centre - half_dimensions, centre + half_dimensions);
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        return self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y;
    }

    pub fn contains(&self, other: &AABB) -> bool {
        return self.min.x <= other.min.x
            && self.max.x >= other.max.x
            && self.min.y <= other.min.y
            && self.max.y >= other.max.y;
    }
}

impl Shape for AABB {
    fn contains_point(&self, point: Vec2f) -> bool {
        return point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y;
    }

    fn intersects_ray(&self, ray: &Segment) -> bool {
        let e1 = Segment::new(
            Vec2f::new(self.min.x, self.min.y),
            Vec2f::new(self.max.x, self.min.y),
        );
        if ray.intersects_ray(&e1) {
            return true;
        }

        let e2 = Segment::new(
            Vec2f::new(self.max.x, self.min.y),
            Vec2f::new(self.max.x, self.max.y),
        );
        if ray.intersects_ray(&e2) {
            return true;
        }

        let e3 = Segment::new(
            Vec2f::new(self.max.x, self.max.y),
            Vec2f::new(self.min.x, self.max.y),
        );
        if ray.intersects_ray(&e3) {
            return true;
        }

        let e4 = Segment::new(
            Vec2f::new(self.min.x, self.max.y),
            Vec2f::new(self.min.x, self.min.y),
        );
        return ray.intersects_ray(&e4);
    }

    fn extents(&self) -> AABB {
        return *self;
    }

    fn area(&self) -> f32 {
        return (self.max.x - self.min.x) * (self.max.y - self.min.y);
    }

    fn centre(&self) -> Vec2f {
        return (self.min + self.max) * 0.5;
    }

    fn translate(&mut self, translation: Vec2f) {
        self.min += translation;
        self.max += translation;
    }

    fn scale(&mut self, scale: Vec2f) {
        self.min *= scale;
        self.max *= scale;
    }

    fn rotate(&mut self, _sin: f32, _cos: f32) {
        // TODO: Maybe this should just do nothing?
        unimplemented!("AABB cannot be rotated")
    }

    fn points(&self) -> &[Vec2f] {
        unimplemented!("AABB does not have points")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersects_test() {
        let aabb_1 = AABB::new(Vec2f::new(0.0, 0.0), Vec2f::new(10.0, 10.0));
        let aabb_2 = AABB::new(Vec2f::new(2.0, 2.0), Vec2f::new(8.0, 8.0));
        let aabb_3 = AABB::new(Vec2f::new(11.0, 11.0), Vec2f::new(12.0, 12.0));

        // intersects
        assert!(aabb_1.intersects(&aabb_2));

        // does not intersect
        assert!(!aabb_1.intersects(&aabb_3));
    }

    #[test]
    fn contains_test() {
        let aabb_1 = AABB::new(Vec2f::new(0.0, 0.0), Vec2f::new(10.0, 10.0));
        let aabb_2 = AABB::new(Vec2f::new(2.0, 2.0), Vec2f::new(8.0, 8.0));

        // contains
        assert!(aabb_1.contains(&aabb_2));

        // does not contain
        assert!(!aabb_2.contains(&aabb_1));
    }

    #[test]
    fn contains_point_test() {
        let aabb = AABB::new(Vec2f::new(0.0, 0.0), Vec2f::new(10.0, 10.0));

        // contains
        assert!(aabb.contains_point(Vec2f::new(2.0, 2.0)));

        // does not contain
        assert!(!aabb.contains_point(Vec2f::new(11.0, 11.0)));
    }

    #[test]
    fn area_test() {
        let aabb = AABB::new(Vec2f::new(0.0, 0.0), Vec2f::new(10.0, 10.0));

        assert_eq!(aabb.area(), 100.0);
    }

    #[test]
    fn centre_test() {
        let aabb = AABB::new(Vec2f::new(0.0, 0.0), Vec2f::new(10.0, 10.0));

        assert_eq!(aabb.centre(), Vec2f::new(5.0, 5.0));
    }
}
