use input::Input;
use maths::{geometry::AABB, linear::Vec2f};
use physics::collider::Collider;
use window::event::KeyCode;

use crate::camera::Camera;

const MOUSE_SENSITIVITY: Vec2f = Vec2f { x: 0.1, y: 0.05 };

pub struct Player {
    pub camera: Camera,
    pub collider: Collider,
    pub bounds: AABB,
    pub sector_index: usize,

    pub velocity: Vec2f,

    pub prev_position: Vec2f,
}

impl Player {
    pub fn new(position: Vec2f, height_offset: f32, collider: Collider) -> Self {
        let camera = Camera::new(position, height_offset);
        let bounds = collider.extents();

        Self {
            camera,
            collider,
            bounds,
            sector_index: 0,

            velocity: Vec2f::ZERO,

            prev_position: position,
        }
    }

    pub fn update(&mut self, delta_seconds: f32, input: &Input) {
        let friction_mag = 175.0;
        let impulse_mag = 250.0;
        let max_speed = 50.0;

        let mut mouse_delta = Vec2f::ZERO;

        if input.mouse.is_grabbed() {
            let mut impulse = Vec2f::ZERO;

            if input.keyboard.is_key_held(KeyCode::W) {
                impulse += self.camera.direction;
            } else if input.keyboard.is_key_held(KeyCode::S) {
                impulse -= self.camera.direction;
            }

            if input.keyboard.is_key_held(KeyCode::A) {
                impulse += self.camera.direction.perpendicular();
            } else if input.keyboard.is_key_held(KeyCode::D) {
                impulse -= self.camera.direction.perpendicular();
            }

            // normalise the direction, and scale by impulse
            impulse = impulse.normalise() * impulse_mag;

            // apply acceleration
            self.velocity += impulse * delta_seconds;

            mouse_delta = input.mouse.get_delta();
            // Negate y-axis to make up positive, as the y-axis is flipped in screen space
            mouse_delta.y = -mouse_delta.y;
        }

        let rot_delta = mouse_delta * MOUSE_SENSITIVITY * delta_seconds;

        // Apply friction
        let friction_impulse = -(self.velocity / self.velocity.magnitude().max(1.0)) * friction_mag;
        self.velocity += friction_impulse * delta_seconds;

        // Clamp velocity to max speed
        let speed = self.velocity.magnitude();
        if speed > max_speed {
            self.velocity /= speed;
            self.velocity *= max_speed;
        }

        // Flush velocity to zero if it's small enough
        if self.velocity.magnitude() < 0.0001 {
            self.velocity = Vec2f::ZERO;
        }

        let pos_delta = self.velocity * delta_seconds;
        self.prev_position = self.camera.position;

        self.collider.translate(pos_delta);
        self.camera.update(pos_delta, rot_delta);
    }
}
