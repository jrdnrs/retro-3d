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
}

impl Player {
    pub fn new(position: Vec2f, height_offset: f32, collider: Collider) -> Self {
        let camera = Camera::new(position, height_offset);
        let bounds = collider.extents();

        Self {
            camera,
            collider,
            bounds,
        }
    }

    pub fn update(&mut self, delta_seconds: f32, input: &Input) {
        if !input.mouse.is_grabbed() {
            return;
        }

        let mut mouse_delta = input.mouse.get_delta();
        // Negate y-axis to make up positive, as the y-axis is flipped in screen space
        mouse_delta.y = -mouse_delta.y;

        let rot_delta = mouse_delta * MOUSE_SENSITIVITY * delta_seconds;
        let mut pos_delta = Vec2f::ZERO;

        if input.keyboard.is_key_held(KeyCode::W) {
            pos_delta += self.camera.direction;
        } else if input.keyboard.is_key_held(KeyCode::S) {
            pos_delta -= self.camera.direction;
        }

        if input.keyboard.is_key_held(KeyCode::A) {
            pos_delta += self.camera.direction.perpendicular();
        } else if input.keyboard.is_key_held(KeyCode::D) {
            pos_delta -= self.camera.direction.perpendicular();
        }

        pos_delta *= delta_seconds * 20.0;

        self.collider.translate(pos_delta);
        self.camera.update(pos_delta, rot_delta);
    }
}
