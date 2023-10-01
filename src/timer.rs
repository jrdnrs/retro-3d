use std::time::{Duration, Instant};

use crate::app::FPS;

#[derive(Debug)]
pub struct Timer {
    pub start: Instant,
    pub frame_time: Duration,
    pub prev_update: Instant,
    pub prev_frame: Instant,
    pub frame_count: usize,

    pub time_buffer: Vec<f32>,
}

impl Timer {
    pub fn new() -> Self {
        let start = Instant::now();
        let frame_time = Duration::from_secs_f32(1.0 / FPS);
        let prev_update = start;
        let prev_frame = start;
        let frame_count = 0;

        Self {
            start,
            frame_time,
            prev_update,
            prev_frame,
            frame_count,

            time_buffer: vec![0.0; 128],
        }
    }

    pub fn delta_seconds(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now - self.prev_update;
        self.prev_update = now;

        delta.as_secs_f32()
    }
}
