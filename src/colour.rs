#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BGRA8 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl BGRA8 {
    pub const BLACK: Self = Self {
        b: 0x00,
        g: 0x00,
        r: 0x00,
        a: 0xFF,
    };
    pub const WHITE: Self = Self {
        b: 0xFF,
        g: 0xFF,
        r: 0xFF,
        a: 0xFF,
    };
    pub const GREY: Self = Self {
        b: 0x80,
        g: 0x80,
        r: 0x80,
        a: 0xFF,
    };
    pub const RED: Self = Self {
        b: 0x00,
        g: 0x00,
        r: 0xFF,
        a: 0xFF,
    };
    pub const ORANGE: Self = Self {
        b: 0x00,
        g: 0xA5,
        r: 0xFF,
        a: 0xFF,
    };
    pub const YELLOW: Self = Self {
        b: 0x00,
        g: 0xFF,
        r: 0xFF,
        a: 0xFF,
    };
    pub const GREEN: Self = Self {
        b: 0x00,
        g: 0xFF,
        r: 0x00,
        a: 0xFF,
    };
    pub const BLUE: Self = Self {
        b: 0xFF,
        g: 0x00,
        r: 0x00,
        a: 0xFF,
    };
    pub const CYAN: Self = Self {
        b: 0xFF,
        g: 0xFF,
        r: 0x00,
        a: 0xFF,
    };
    pub const PURPLE: Self = Self {
        b: 0x80,
        g: 0x00,
        r: 0x80,
        a: 0xFF,
    };
    pub const MAGENTA: Self = Self {
        b: 0xFF,
        g: 0x00,
        r: 0xFF,
        a: 0xFF,
    };

    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            b: blue,
            g: green,
            r: red,
            a: alpha,
        }
    }

    /// Blends two colours together using the alpha value of the first colour and 1.0 - alpha of the
    /// second colour.
    pub fn blend(self, other: Self) -> Self {
        let alpha = self.a as usize;
        let beta = 255 - alpha;

        let r1 = self.r as usize * alpha;
        let g1 = self.g as usize * alpha;
        let b1 = self.b as usize * alpha;

        let r2 = other.r as usize * beta;
        let g2 = other.g as usize * beta;
        let b2 = other.b as usize * beta;

        let r = ((r1 + r2) >> 8) as u8;
        let g = ((g1 + g2) >> 8) as u8;
        let b = ((b1 + b2) >> 8) as u8;

        Self { b, g, r, a: 0xFF }
    }

    /// Multiplies the colour by the given value (effectively between 0 and 1)
    pub fn darken(self, d: u8) -> Self {
        let d = d as usize;

        let r = (self.r as usize * d) >> 8;
        let g = (self.g as usize * d) >> 8;
        let b = (self.b as usize * d) >> 8;

        Self {
            b: b as u8,
            g: g as u8,
            r: r as u8,
            a: self.a,
        }
    }

    /// Multiplies the colour by the given value (where 0xFF is 1.0), and saturates the result.
    pub fn lighten(self, l: usize) -> Self {
        let r = ((self.r as usize * l) >> 8).max(0xFF);
        let g = ((self.g as usize * l) >> 8).max(0xFF);
        let b = ((self.b as usize * l) >> 8).max(0xFF);

        Self {
            b: b as u8,
            g: g as u8,
            r: r as u8,
            a: self.a,
        }
    }

    pub fn as_u32(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }
}

impl From<RGB8> for BGRA8 {
    fn from(rgb: RGB8) -> Self {
        Self {
            b: rgb.b,
            g: rgb.g,
            r: rgb.r,
            a: 0xFF,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB8 {
    pub const BLACK: Self = Self {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    };
    pub const WHITE: Self = Self {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    };
    pub const GREY: Self = Self {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    };
    pub const RED: Self = Self {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    };
    pub const ORANGE: Self = Self {
        r: 0xFF,
        g: 0xA5,
        b: 0x00,
    };
    pub const YELLOW: Self = Self {
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    };
    pub const GREEN: Self = Self {
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    };
    pub const BLUE: Self = Self {
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    };
    pub const CYAN: Self = Self {
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    };
    pub const PURPLE: Self = Self {
        r: 0x80,
        g: 0x00,
        b: 0x80,
    };
    pub const MAGENTA: Self = Self {
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    };

    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
        }
    }

    pub fn blend(self, other: Self, alpha: u8) -> Self {
        let alpha = alpha as u16;
        let beta = 255 - alpha;

        let r1 = self.r as u16 * alpha;
        let g1 = self.g as u16 * alpha;
        let b1 = self.b as u16 * alpha;

        let r2 = other.r as u16 * beta;
        let g2 = other.g as u16 * beta;
        let b2 = other.b as u16 * beta;

        let r = ((r1 + r2) >> 8) as u8;
        let g = ((g1 + g2) >> 8) as u8;
        let b = ((b1 + b2) >> 8) as u8;

        Self { r, g, b }
    }

    pub fn as_u32(self) -> u32 {
        unsafe { std::mem::transmute([self.r, self.g, self.b, 0xFF]) }
    }
}
