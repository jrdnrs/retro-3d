use maths::linear::Vec2f;

use crate::colour::BGRA8;

pub struct Framebuffer {
    width: usize,
    height: usize,
    half_width: f32,
    half_height: f32,
    aspect_ratio: f32,
    pixels: Vec<BGRA8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        let pixels = vec![BGRA8::default(); len];

        Self {
            width,
            height,
            half_width: width as f32 * 0.5,
            half_height: height as f32 * 0.5,
            aspect_ratio: width as f32 / height as f32,
            pixels,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn half_width(&self) -> f32 {
        self.half_width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn half_height(&self) -> f32 {
        self.half_height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn pixels(&self) -> &[BGRA8] {
        &self.pixels
    }

    pub fn pixels_as_u32(&self) -> &[u32] {
        unsafe { core::mem::transmute(&self.pixels as &[BGRA8]) }
    }

    pub fn pixels_as_u8(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.pixels.as_ptr() as *const u8,
                self.pixels.len() * core::mem::size_of::<BGRA8>(),
            )
        }
    }

    pub fn fill(&mut self, colour: BGRA8) {
        self.pixels.fill(colour);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: BGRA8) {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        self.pixels[index] = colour;
    }

    pub unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, colour: BGRA8) {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        debug_assert!(index < self.pixels.len());
        *self.pixels.get_unchecked_mut(index) = colour;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> BGRA8 {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        self.pixels[index]
    }

    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> BGRA8 {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        debug_assert!(index < self.pixels.len());
        *self.pixels.get_unchecked(index)
    }

    pub fn blend_pixel(&mut self, x: usize, y: usize, colour: BGRA8) {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        let blended = self.pixels[index].blend(colour);
        self.pixels[index] = blended;
    }

    pub unsafe fn blend_pixel_unchecked(&mut self, x: usize, y: usize, colour: BGRA8) {
        debug_assert!(x < self.width && y < self.height);
        let index = (y * self.width) + x;

        debug_assert!(index < self.pixels.len());
        let blended = self.pixels.get_unchecked(index).blend(colour);
        *self.pixels.get_unchecked_mut(index) = blended;
    }

    /// # Deprecated
    pub fn draw_line(&mut self, mut start: Vec2f, mut end: Vec2f, colour: BGRA8) {
        start.x = start.x.clamp(0.0, (self.width - 1) as f32);
        start.y = start.y.clamp(0.0, (self.height - 1) as f32);
        end.x = end.x.clamp(0.0, (self.width - 1) as f32);
        end.y = end.y.clamp(0.0, (self.height - 1) as f32);

        let delta = end - start;
        let steps = delta.x.abs().max(delta.y.abs());
        let increment = delta / steps;

        let mut position = start;

        for _ in 0..steps as usize {
            let x = position.x.round() as usize;
            let y = position.y.round() as usize;

            self.set_pixel(x, y, colour);
            position += increment;
        }
    }
}
