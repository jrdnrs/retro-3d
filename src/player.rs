use input::Input;
use maths::{
    geometry::{Circle, Shape},
    linear::Vec2f,
};
use window::event::KeyCode;

use crate::camera::Camera;

const MOUSE_SENSITIVITY: Vec2f = Vec2f { x: 0.1, y: 0.05 };

pub struct Player {
    pub camera: Camera,
    pub sector_index: usize,
    pub collider: Circle,
    pub prev_position: Vec2f,
    pub velocity: Vec2f,

    pub crouch: bool,
    pub head_z: f32,
    pub knee_z: f32,
}

impl Player {
    pub fn new(position: Vec2f, z: f32, sector_index: usize) -> Self {
        let camera = Camera::new(position, z);
        let collider = Circle::new(position, 10.0);

        let height = 15.0;
        let head_z = z + height * 0.2;
        let knee_z = z - height * 0.6;

        Self {
            camera,
            sector_index,
            collider,
            prev_position: position,
            velocity: Vec2f::ZERO,

            crouch: false,
            head_z,
            knee_z,
        }
    }

    pub fn update_movement(&mut self, delta_seconds: f32, input: &Input) {
        let friction_mag = 175.0;
        let impulse_mag = 300.0;
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

            mouse_delta = input.mouse.delta();
            // Negate y-axis to make up positive, as the y-axis is flipped in screen space
            mouse_delta.y = -mouse_delta.y;
        }

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

        let translation = self.velocity * delta_seconds;
        let rotation = mouse_delta * MOUSE_SENSITIVITY * delta_seconds;

        self.rotate(rotation);
        self.translate(translation);

        if input.keyboard.is_key_pressed(KeyCode::ShiftLeft) {
            self.toggle_crouch();
        }
    }

    pub fn toggle_crouch(&mut self) {
        self.crouch = !self.crouch;

        if self.crouch {
            
        } else {
        }
    }

    pub fn translate(&mut self, translation: Vec2f) {
        self.prev_position = self.camera.position;
        self.collider.translate(translation);
        self.camera.translate(translation);
    }

    pub fn rotate(&mut self, rotation: Vec2f) {
        self.camera.rotate(rotation);
    }
}
