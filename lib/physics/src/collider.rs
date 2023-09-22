use maths::{
    geometry::{Circle, Polygon, Segment, Shape, Triangle, AABB},
    linear::Vec2f,
};

pub enum Collider {
    AABB(AABB),
    Circle(Circle),
    Polygon(Polygon),
    Segment(Segment),
    Triangle(Triangle),
}

impl Collider {
    pub fn new_aabb(aabb: AABB) -> Self {
        Self::AABB(aabb)
    }

    pub fn new_circle(circle: Circle) -> Self {
        Self::Circle(circle)
    }

    pub fn new_polygon(polygon: Polygon) -> Self {
        Self::Polygon(polygon)
    }

    pub fn new_segment(segment: Segment) -> Self {
        Self::Segment(segment)
    }

    pub fn new_triangle(triangle: Triangle) -> Self {
        Self::Triangle(triangle)
    }

    pub fn contains_point(&self, point: Vec2f) -> bool {
        match self {
            Collider::AABB(aabb) => aabb.contains_point(point),
            Collider::Circle(circle) => circle.contains_point(point),
            Collider::Polygon(polygon) => polygon.contains_point(point),
            Collider::Segment(segment) => segment.contains_point(point),
            Collider::Triangle(triangle) => triangle.contains_point(point),
        }
    }

    pub fn intersects_ray(&self, ray: &Segment) -> bool {
        match self {
            Collider::AABB(aabb) => aabb.intersects_ray(ray),
            Collider::Circle(circle) => circle.intersects_ray(ray),
            Collider::Polygon(polygon) => polygon.intersects_ray(ray),
            Collider::Segment(segment) => segment.intersects_ray(ray),
            Collider::Triangle(triangle) => triangle.intersects_ray(ray),
        }
    }

    pub fn extents(&self) -> AABB {
        match self {
            Collider::AABB(aabb) => aabb.extents(),
            Collider::Circle(circle) => circle.extents(),
            Collider::Polygon(polygon) => polygon.extents(),
            Collider::Segment(segment) => segment.extents(),
            Collider::Triangle(triangle) => triangle.extents(),
        }
    }

    pub fn area(&self) -> f32 {
        match self {
            Collider::AABB(aabb) => aabb.area(),
            Collider::Circle(circle) => circle.area(),
            Collider::Polygon(polygon) => polygon.area(),
            Collider::Segment(segment) => segment.area(),
            Collider::Triangle(triangle) => triangle.area(),
        }
    }

    pub fn centre(&self) -> Vec2f {
        match self {
            Collider::AABB(aabb) => aabb.centre(),
            Collider::Circle(circle) => circle.centre(),
            Collider::Polygon(polygon) => polygon.centre(),
            Collider::Segment(segment) => segment.centre(),
            Collider::Triangle(triangle) => triangle.centre(),
        }
    }

    pub fn translate(&mut self, translation: Vec2f) {
        match self {
            Collider::AABB(aabb) => aabb.translate(translation),
            Collider::Circle(circle) => circle.translate(translation),
            Collider::Polygon(polygon) => polygon.translate(translation),
            Collider::Segment(segment) => segment.translate(translation),
            Collider::Triangle(triangle) => triangle.translate(translation),
        }
    }

    pub fn scale(&mut self, scale: Vec2f) {
        match self {
            Collider::AABB(aabb) => aabb.scale(scale),
            Collider::Circle(circle) => circle.scale(scale),
            Collider::Polygon(polygon) => polygon.scale(scale),
            Collider::Segment(segment) => segment.scale(scale),
            Collider::Triangle(triangle) => triangle.scale(scale),
        }
    }

    pub fn rotate(&mut self, sin: f32, cos: f32) {
        match self {
            Collider::AABB(aabb) => aabb.rotate(sin, cos),
            Collider::Circle(circle) => circle.rotate(sin, cos),
            Collider::Polygon(polygon) => polygon.rotate(sin, cos),
            Collider::Segment(segment) => segment.rotate(sin, cos),
            Collider::Triangle(triangle) => triangle.rotate(sin, cos),
        }
    }
}
