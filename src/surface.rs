use maths::linear::{Mat2f, Vec2f};

#[derive(Clone, Copy, Debug)]
pub struct WallTexture {
    pub index: usize,
    pub offset: Vec2f,
    pub scale: Vec2f,
}

impl WallTexture {
    pub fn new(index: usize, offset: Vec2f, scale: Vec2f) -> Self {
        Self {
            index,
            offset,
            scale,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaneTexture {
    pub index: usize,
    pub offset: Vec2f,
    pub scale_rotate: Mat2f,
}

impl PlaneTexture {
    pub fn new(index: usize, offset: Vec2f, scale: Vec2f, rotate: f32) -> Self {
        Self {
            index,
            offset,
            scale_rotate: Mat2f::rotation(rotate) * Mat2f::scale(scale),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Portal {
    pub sector: usize,
    pub upper_texture: WallTexture,
    pub lower_texture: WallTexture,
}

impl Portal {
    pub fn new(sector: usize, upper_texture: WallTexture, lower_texture: WallTexture) -> Self {
        Self {
            sector,
            upper_texture,
            lower_texture,
        }
    }
}

#[derive(Debug)]
pub struct Sector {
    pub walls: Vec<Wall>,
    pub floor: Plane,
    pub ceiling: Plane,
}

#[derive(Debug)]
pub struct Wall {
    pub a: Vec2f,
    pub b: Vec2f,
    pub width: f32,
    pub texture_data: WallTexture,
    pub portal: Option<Portal>,
}

impl Wall {
    pub fn new(a: Vec2f, b: Vec2f, texture_data: WallTexture, portal: Option<Portal>) -> Self {
        Self {
            a,
            b,
            width: (b - a).magnitude(),
            texture_data,
            portal,
        }
    }
}

#[derive(Debug)]
pub struct Plane {
    pub height: f32,
    pub texture_data: PlaneTexture,
}

impl Plane {
    pub fn new(height: f32, texture_data: PlaneTexture) -> Self {
        Self {
            height,
            texture_data,
        }
    }
}


#[derive(Debug)]
pub struct Sector {
    pub walls: Vec<Wall>,
    pub floor: Plane,
    pub ceiling: Plane,
}
