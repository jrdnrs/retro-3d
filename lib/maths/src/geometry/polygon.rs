use crate::linear::Vec2f;

use super::{Segment, Shape, AABB};


const INFINITY: f32 = 1e30;


pub struct Polygon {
    pub vertices: Vec<Vec2f>,
}

impl Polygon {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    pub fn from_vertices(vertices: Vec<Vec2f>) -> Self {
        Self { vertices }
    }

    pub fn from_rect(centre: Vec2f, dimensions: Vec2f) -> Self {
        let half_dimensions = dimensions * 0.5;

        return Self::from_vertices(vec![
            centre + Vec2f::new(-half_dimensions.x, -half_dimensions.y),
            centre + Vec2f::new(half_dimensions.x, -half_dimensions.y),
            centre + Vec2f::new(half_dimensions.x, half_dimensions.y),
            centre + Vec2f::new(-half_dimensions.x, half_dimensions.y),
        ]);
    }
}

impl Shape for Polygon {
    fn contains_point(&self, point: Vec2f) -> bool {
        let ray = Segment::new(point, Vec2f::new(INFINITY, point.y));

        let mut inside = false;

        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();

            let edge = Segment::new(self.vertices[i], self.vertices[j]);

            if edge.intersects(&ray) {
                inside = !inside;
            }
        }

        return inside;
    }

    fn intersects_ray(&self, ray: &Segment) -> bool {
        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();

            let edge = Segment::new(self.vertices[i], self.vertices[j]);

            if edge.intersects(ray) {
                return true;
            }
        }

        return false;
    }

    fn extents(&self) -> AABB {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;

        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for vertex in self.vertices.iter() {
            min_x = min_x.min(vertex.x);
            min_y = min_y.min(vertex.y);

            max_x = max_x.max(vertex.x);
            max_y = max_y.max(vertex.y);
        }

        return AABB::new(Vec2f::new(min_x, min_y), Vec2f::new(max_x, max_y));
    }

    fn area(&self) -> f32 {
        let mut area = 0.0;

        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();

            area += self.vertices[i].cross(self.vertices[j]);
        }

        return area.abs() * 0.5;
    }

    fn centre(&self) -> Vec2f {
        let mut centre = Vec2f::ZERO;

        for vertex in self.vertices.iter() {
            centre += *vertex;
        }

        return centre / self.vertices.len() as f32;
    }

    fn translate(&mut self, translation: Vec2f) {
        for vertex in self.vertices.iter_mut() {
            *vertex += translation;
        }
    }

    fn scale(&mut self, scale: Vec2f) {
        for vertex in self.vertices.iter_mut() {
            *vertex *= scale;
        }
    }

    fn rotate(&mut self, sin: f32, cos: f32) {
        for vertex in self.vertices.iter_mut() {
            *vertex = vertex.rotate(sin, cos);
        }
    }

    fn points(&self) -> &[Vec2f] {
        return &self.vertices;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_point_test() {
        let polygon = Polygon::from_vertices(vec![
            Vec2f::new(0.0, 0.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(1.0, 1.0),
            Vec2f::new(0.0, 1.0),
        ]);

        // contains
        assert!(polygon.contains_point(Vec2f::new(0.5, 0.5)));

        // does not contain
        assert!(!polygon.contains_point(Vec2f::new(1.5, 0.5)));
    }

    #[test]
    fn intersects_ray_test() {
        let polygon = Polygon::from_vertices(vec![
            Vec2f::new(0.0, 0.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(1.0, 1.0),
            Vec2f::new(0.0, 1.0),
        ]);

        // intersects
        assert!(polygon.intersects_ray(&Segment::new(Vec2f::new(0.5, 0.5), Vec2f::new(2.0, 1.0))));

        // does not intersect
        assert!(!polygon.intersects_ray(&Segment::new(Vec2f::new(1.5, 0.5), Vec2f::new(3.0, 0.0))));
    }
}
