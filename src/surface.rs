use maths::linear::{Mat2f, Vec2f};

#[derive(Debug)]
pub struct Wall {
    pub a: Vec2f,
    pub b: Vec2f,
    pub width: f32,
    pub texture_index: usize,
    pub texture_offset: Vec2f,
    pub texture_scale: Vec2f,
    pub portal: Option<usize>,
}

impl Wall {
    pub fn new(
        a: Vec2f,
        b: Vec2f,
        texture_index: usize,
        texture_offset: Vec2f,
        texture_scale: Vec2f,
        portal: Option<usize>,
    ) -> Self {
        Self {
            a,
            b,
            width: (b - a).magnitude(),
            texture_index,
            texture_offset,
            texture_scale,
            portal,
        }
    }
}

#[derive(Debug)]
pub struct Plane {
    pub height: f32,
    pub texture_index: usize,
    pub texture_offset: Vec2f,
    pub texture_scale_rotate: Mat2f,
}

impl Plane {
    pub fn new(
        height: f32,
        texture_index: usize,
        texture_offset: Vec2f,
        texture_scale_rotate: Mat2f,
    ) -> Self {
        Self {
            height,
            texture_index,
            texture_offset,
            texture_scale_rotate,
        }
    }
}

#[derive(Debug)]
pub struct Sector {
    pub walls: Vec<Wall>,
    pub floor: Plane,
    pub ceiling: Plane,
}
