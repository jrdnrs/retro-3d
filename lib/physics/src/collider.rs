use maths::{
    geometry::{Circle, Segment, Shape, AABB},
    linear::Vec2f,
};

use crate::collision::{
    collision_circles, sat_collision_circle_polygon, sat_collision_polygons, Collision,
};

pub enum Collider {
    AABB(AABB),
    Circle(Circle),
    Segment(Segment),
}

impl Collider {
    pub fn from_aabb(aabb: AABB) -> Self {
        Self::AABB(aabb)
    }

    pub fn from_circle(circle: Circle) -> Self {
        Self::Circle(circle)
    }

    pub fn from_segment(segment: Segment) -> Self {
        Self::Segment(segment)
    }

    pub fn contains_point(&self, point: Vec2f) -> bool {
        match self {
            Collider::AABB(aabb) => aabb.contains_point(point),
            Collider::Circle(circle) => circle.contains_point(point),
            Collider::Segment(segment) => segment.contains_point(point),
        }
    }

    pub fn intersects(&self, other: &Collider) -> Option<Collision> {
        match (self, other) {
            (Collider::Circle(a), Collider::Circle(b)) => collision_circles(a, b),

            (Collider::Circle(circle), other) | (other, Collider::Circle(circle)) => {
                sat_collision_circle_polygon(circle, other.points())
            }

            _ => sat_collision_polygons(self.points(), other.points()),
        }
    }

    pub fn intersects_ray(&self, ray: &Segment) -> bool {
        match self {
            Collider::AABB(aabb) => aabb.intersects_ray(ray),
            Collider::Circle(circle) => circle.intersects_ray(ray),
            Collider::Segment(segment) => segment.intersects_ray(ray),
        }
    }

    pub fn extents(&self) -> AABB {
        match self {
            Collider::AABB(aabb) => aabb.extents(),
            Collider::Circle(circle) => circle.extents(),
            Collider::Segment(segment) => segment.extents(),
        }
    }

    pub fn area(&self) -> f32 {
        match self {
            Collider::AABB(aabb) => aabb.area(),
            Collider::Circle(circle) => circle.area(),
            Collider::Segment(segment) => segment.area(),
        }
    }

    pub fn centre(&self) -> Vec2f {
        match self {
            Collider::AABB(aabb) => aabb.centre(),
            Collider::Circle(circle) => circle.centre(),
            Collider::Segment(segment) => segment.centre(),
        }
    }

    pub fn translate(&mut self, translation: Vec2f) {
        match self {
            Collider::AABB(aabb) => aabb.translate(translation),
            Collider::Circle(circle) => circle.translate(translation),
            Collider::Segment(segment) => segment.translate(translation),
        }
    }

    pub fn scale(&mut self, scale: Vec2f) {
        match self {
            Collider::AABB(aabb) => aabb.scale(scale),
            Collider::Circle(circle) => circle.scale(scale),
            Collider::Segment(segment) => segment.scale(scale),
        }
    }

    pub fn rotate(&mut self, sin: f32, cos: f32) {
        match self {
            Collider::AABB(aabb) => aabb.rotate(sin, cos),
            Collider::Circle(circle) => circle.rotate(sin, cos),
            Collider::Segment(segment) => segment.rotate(sin, cos),
        }
    }

    pub fn points(&self) -> &[Vec2f] {
        match self {
            Collider::AABB(aabb) => aabb.points(),
            Collider::Circle(circle) => circle.points(),
            Collider::Segment(segment) => segment.points(),
        }
    }
}
