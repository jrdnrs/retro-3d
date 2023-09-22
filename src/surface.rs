use maths::linear::Vec2f;

#[derive(Debug)]
pub struct Wall {
    pub a: Vec2f,
    pub b: Vec2f,
    pub texture_basis: Basis,
    pub texture_index: usize,
    pub portal: Option<usize>,
}

impl Wall {
    pub fn new(
        a: Vec2f,
        b: Vec2f,
        texture_basis: Basis,
        texture_index: usize,
        portal: Option<usize>,
    ) -> Self {
        Self {
            a,
            b,
            texture_basis,
            texture_index,
            portal,
        }
    }
}

#[derive(Debug)]
pub struct Plane {
    pub height: f32,
    pub texture_basis: Basis,
    pub texture_index: usize,
}

impl Plane {
    pub fn new(height: f32, texture_basis: Basis, texture_index: usize) -> Self {
        Self {
            height,
            texture_basis,
            texture_index,
        }
    }
}

#[derive(Debug)]
pub struct Sector {
    pub walls: Vec<Wall>,
    pub floor: Plane,
    pub ceiling: Plane,
}

#[derive(Clone, Copy, Debug)]
pub struct Basis {
    pub offset: Vec2f,
    pub scale: Vec2f,
}

impl Basis {
    pub const IDENTITY: Self = Self {
        offset: Vec2f::ZERO,
        scale: Vec2f::ONE,
    };

    pub fn new(offset: Vec2f, scale: Vec2f) -> Self {
        Self { offset, scale }
    }

    pub fn apply(&self, p: Vec2f) -> Vec2f {
        self.scale * p + self.offset
    }
}
