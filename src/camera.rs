use std::f32::consts::FRAC_PI_4;

use maths::linear::Vec2f;

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vec2f,
    pub height_offset: f32,
    pub direction: Vec2f,

    pub yaw: f32,
    pub yaw_sin: f32,
    pub yaw_cos: f32,
    pub pitch: f32,
    pub pitch_tan: f32,
}

impl Camera {
    pub fn new(position: Vec2f, height_offset: f32) -> Self {
        let direction = Vec2f::new(0.0, 1.0);
        let yaw = 0.0;
        let yaw_sin = 0.0;
        let yaw_cos = 1.0;
        let pitch = 0.0;
        let pitch_tan = 0.0;

        Self {
            position,
            height_offset,
            direction,

            yaw,
            yaw_sin,
            yaw_cos,
            pitch,
            pitch_tan,
        }
    }

    pub fn update(&mut self, pos_delta: Vec2f, rot_delta: Vec2f) {
        self.position += pos_delta;

        self.yaw += rot_delta.x;
        (self.yaw_sin, self.yaw_cos) = self.yaw.sin_cos();

        // Clamps pitch to +-45 degrees
        self.pitch = (self.pitch + rot_delta.y).clamp(-FRAC_PI_4, FRAC_PI_4);
        self.pitch_tan = self.pitch.tan();

        self.direction = Vec2f::new(self.yaw_sin, self.yaw_cos);
    }
}
