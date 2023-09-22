#[derive(Clone, Copy, Debug, Default)]
pub struct Colour {
    pub rgb: u32,
}

impl Colour {
    pub const BLACK: Self = Self { rgb: 0x00_00_00 };
    pub const WHITE: Self = Self { rgb: 0xFF_FF_FF };
    pub const GREY: Self = Self { rgb: 0x80_80_80 };
    pub const RED: Self = Self { rgb: 0xFF_00_00 };
    pub const ORANGE: Self = Self { rgb: 0xFF_A5_00 };
    pub const YELLOW: Self = Self { rgb: 0xFF_FF_00 };
    pub const GREEN: Self = Self { rgb: 0x00_FF_00 };
    pub const BLUE: Self = Self { rgb: 0x00_00_FF };
    pub const CYAN: Self = Self { rgb: 0x00_FF_FF };
    pub const PURPLE: Self = Self { rgb: 0x80_00_80 };
    pub const MAGENTA: Self = Self { rgb: 0xFF_00_FF };

    pub fn new(rgb: u32) -> Self {
        Self { rgb }
    }

    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgb: ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }

    pub fn to_u8(&self) -> (u8, u8, u8) {
        let r = (self.rgb >> 16) as u8;
        let g = (self.rgb >> 8) as u8;
        let b = self.rgb as u8;

        (r, g, b)
    }

    pub fn blend(&self, other: Self, alpha: u8) -> Self {
        let alpha = alpha as u16;
        let beta = 255 - alpha;

        let (r1, g1, b1) = self.to_u8();

        let r1 = r1 as u16 * alpha;
        let g1 = g1 as u16 * alpha;
        let b1 = b1 as u16 * alpha;

        let (r2, g2, b2) = other.to_u8();

        let r2 = r2 as u16 * beta;
        let g2 = g2 as u16 * beta;
        let b2 = b2 as u16 * beta;

        // TODO: Test performance of bitshift vs division
        let r = ((r1 + r2) >> 8) as u8;
        let g = ((g1 + g2) >> 8) as u8;
        let b = ((b1 + b2) >> 8) as u8;

        Self::from_u8(r, g, b)
    }

}
