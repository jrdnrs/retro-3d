use crate::linear::{Vec2f, Vec3f};

use super::{aabb::AABB, segment::Segment, shape::Shape};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Vec2f,
    pub b: Vec2f,
    pub c: Vec2f,
}

impl Triangle {
    pub fn new(a: Vec2f, b: Vec2f, c: Vec2f) -> Self {
        Self { a, b, c }
    }

    pub fn barycentric_from_inv_area(&self, point: Vec2f, inv_area: f32) -> Vec3f {
        let area_bcp = Triangle::new(self.b, self.c, point).area();
        let area_acp = Triangle::new(self.a, self.c, point).area();
        let area_abp = Triangle::new(self.a, self.b, point).area();

        let u = area_bcp * inv_area;
        let v = area_acp * inv_area;
        let w = area_abp * inv_area;

        return Vec3f::new(u, v, w);
    }

    pub fn barycentric(&self, point: Vec2f) -> Vec3f {
        let area = self.area();
        let inv_area = 1.0 / area;

        return self.barycentric_from_inv_area(point, inv_area);
    }
}

impl Shape for Triangle {
    fn contains_point(&self, point: Vec2f) -> bool {
        // pineda's method (same-side technique)
        let edge_1 = Segment::new(self.b, self.a).edge_side(point);
        let edge_2 = Segment::new(self.c, self.b).edge_side(point);
        let edge_3 = Segment::new(self.a, self.c).edge_side(point);

        // SAFETY: This is just transmuting to get the sign bit, it's fine.
        let sign_1 = unsafe { core::mem::transmute::<f32, u32>(edge_1) & 0x8000_0000 };
        let sign_2 = unsafe { core::mem::transmute::<f32, u32>(edge_2) & 0x8000_0000 };
        let sign_3 = unsafe { core::mem::transmute::<f32, u32>(edge_3) & 0x8000_0000 };

        return sign_1 == sign_2 && sign_2 == sign_3;
    }

    fn intersects_ray(&self, ray: &Segment) -> bool {
        let edge_1 = Segment::new(self.b, self.a);
        if edge_1.intersects(ray) {
            return true;
        }

        let edge_2 = Segment::new(self.c, self.b);
        if edge_2.intersects(ray) {
            return true;
        }

        let edge_3 = Segment::new(self.a, self.c);
        return edge_3.intersects(ray);
    }

    fn extents(&self) -> AABB {
        let min_x = self.a.x.min(self.b.x).min(self.c.x);
        let min_y = self.a.y.min(self.b.y).min(self.c.y);

        let max_x = self.a.x.max(self.b.x).max(self.c.x);
        let max_y = self.a.y.max(self.b.y).max(self.c.y);

        return AABB::new(Vec2f::new(min_x, min_y), Vec2f::new(max_x, max_y));
    }

    fn area(&self) -> f32 {
        let parallelogram_area = (self.b - self.a).cross(self.c - self.a).abs();
        return parallelogram_area * 0.5;
    }

    fn centre(&self) -> Vec2f {
        let x = (self.a.x + self.b.x + self.c.x) / 3.0;
        let y = (self.a.y + self.b.y + self.c.y) / 3.0;

        return Vec2f::new(x, y);
    }

    fn translate(&mut self, translation: Vec2f) {
        self.a += translation;
        self.b += translation;
        self.c += translation;
    }

    fn scale(&mut self, scale: Vec2f) {
        self.a *= scale;
        self.b *= scale;
        self.c *= scale;
    }

    fn rotate(&mut self, sin: f32, cos: f32) {
        self.a = self.a.rotate(sin, cos);
        self.b = self.b.rotate(sin, cos);
        self.c = self.c.rotate(sin, cos);
    }

    fn points(&self) -> &[Vec2f] {
        return unsafe { std::slice::from_raw_parts(&self.a as *const _, 3) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_point() {
        let triangle = Triangle::new(
            Vec2f::new(0.0, 0.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(0.0, 1.0),
        );

        // inside
        assert!(triangle.contains_point(Vec2f::new(0.2, 0.2)));

        // outside
        assert!(!triangle.contains_point(Vec2f::new(1.0, 1.0)));
    }

    #[test]
    fn intersects_ray_test() {
        let triangle = Triangle::new(
            Vec2f::new(0.0, 0.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(0.0, 1.0),
        );

        // intersects
        assert!(triangle.intersects_ray(&Segment::new(Vec2f::new(0.1, 0.1), Vec2f::new(2.0, 2.0))));

        // does not intersect
        assert!(!triangle.intersects_ray(&Segment::new(Vec2f::new(2.0, 2.0), Vec2f::new(4.0, 4.0))));
    }

    #[test]
    fn test_area() {
        let triangle = Triangle::new(
            Vec2f::new(0.0, 0.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(0.0, 1.0),
        );

        assert_eq!(triangle.area(), 0.5);

        let triangle = Triangle::new(
            Vec2f::new(1.0, 0.0),
            Vec2f::new(2.5, 1.0),
            Vec2f::new(0.0, 4.0),
        );

        assert_eq!(triangle.area(), 3.5);
    }
}
