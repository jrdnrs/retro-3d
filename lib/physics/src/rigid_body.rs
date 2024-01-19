use maths::{
    geometry::{Shape, AABB},
    linear::Vec2f,
};

use crate::collider::Collider;

pub struct RigidBody {
    pub id: usize,

    pub collider: Collider,
    pub bounds: AABB,
    pub fixed: bool,
    pub mass: f32,
    pub inv_mass: f32,

    pub velocity: Vec2f,
    pub position: Vec2f,
}

impl RigidBody {
    pub fn new(collider: Collider, mass: f32, fixed: bool, id: usize) -> Self {
        let bounds = collider.extents();
        let inv_mass = if fixed { 0.0 } else { 1.0 / mass };

        Self {
            id,
            collider,
            bounds,
            fixed,
            mass,
            inv_mass,
            velocity: Vec2f::ZERO,
            position: Vec2f::ZERO,
        }
    }

    pub fn translate(&mut self, translation: Vec2f) {
        self.position += translation;
        self.collider.translate(translation);
        self.bounds.translate(translation);
    }

    pub fn integrate_position(&mut self, delta_seconds: f32) {
        let translation = self.velocity * delta_seconds;

        self.translate(translation);
    }
}
