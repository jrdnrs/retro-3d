use crate::linear::Vec2f;

use super::{aabb::AABB, segment::Segment};

pub trait Shape: Sized {
    fn contains_point(&self, point: Vec2f) -> bool;
    fn intersects_ray(&self, ray: &Segment) -> bool;
    fn overlaps(&self, other: &impl Shape) -> bool {
        super::sat::separating_axis_test(self.points(), other.points())
    }
    fn extents(&self) -> AABB;
    fn area(&self) -> f32;
    fn centre(&self) -> Vec2f;
    fn translate(&mut self, translation: Vec2f);
    fn rotate(&mut self, sin: f32, cos: f32);
    fn scale(&mut self, scale: Vec2f);
    fn points(&self) -> &[Vec2f];
}
