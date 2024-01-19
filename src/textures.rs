use std::path::Path;

use crate::{bitmap::Bitmap, colour::BGRA8, consts::MIP_LEVELS};

#[derive(Clone, Copy, Debug, Default)]
pub struct MipLevel {
    pub width: usize,
    pub height: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Texture {
    pub levels: [MipLevel; MIP_LEVELS],
    pub pixels: Vec<BGRA8>,
}

impl Texture {
    pub fn from_path_png(path: impl AsRef<Path>) -> Result<Self, &'static str> {
        let bitmap = Bitmap::from_path_png(path)?;
        Ok(Self::from_bitmap(bitmap))
    }

    fn from_bitmap(bitmap: Bitmap) -> Self {
        let levels = Self::calculate_mip_levels(&bitmap);
        let buffer_size = levels[MIP_LEVELS - 1].offset
            + levels[MIP_LEVELS - 1].width * levels[MIP_LEVELS - 1].height;

        let mut pixels = vec![BGRA8::default(); buffer_size];

        // Copy the pixels from the bitmap into the first level of the texture
        pixels[..bitmap.pixels().len()].copy_from_slice(bitmap.pixels());

        Self::generate_mip_maps(&levels, &mut pixels);

        Self { levels, pixels }
    }

    fn calculate_mip_levels(bitmap: &Bitmap) -> [MipLevel; MIP_LEVELS] {
        let mut mip_width = bitmap.width();
        let mut mip_height = bitmap.height();
        let mut offset = 0;

        core::array::from_fn(|_| {
            let level = MipLevel {
                width: mip_width,
                height: mip_height,
                offset,
            };

            offset += mip_width * mip_height;
            mip_width /= 2;
            mip_height /= 2;

            level
        })
    }

    /// Generates mip maps for the given texture, assuming that the first level is already filled
    fn generate_mip_maps(levels: &[MipLevel], buffer: &mut [BGRA8]) {
        for i in 1..MIP_LEVELS {
            let src_width = levels[i - 1].width;
            let src_height = levels[i - 1].height;
            let read_index = levels[i - 1].offset;
            let write_index = levels[i].offset;

            // `src` is slice of the pixels from the previous level, and `dst` is current level
            let (src, dst) = buffer.split_at_mut(write_index);
            let src = &src[read_index..];

            downscale_3x3_box_filter(src, src_width, src_height, dst);
        }
    }

    pub fn sample(&self, x: usize, y: usize, level: usize) -> BGRA8 {
        debug_assert!(x < self.levels[level].width && y < self.levels[level].height);

        let local_offset = y * self.levels[level].width + x;
        let global_offset = self.levels[level].offset + local_offset;

        self.pixels[global_offset]
    }

    pub unsafe fn sample_unchecked(&self, x: usize, y: usize, level: usize) -> BGRA8 {
        debug_assert!(x < self.levels[level].width && y < self.levels[level].height);

        debug_assert!(level < MIP_LEVELS);
        let local_offset = y * self.levels.get_unchecked(level).width + x;
        let global_offset = self.levels.get_unchecked(level).offset + local_offset;

        debug_assert!(global_offset < self.pixels.len());
        *self.pixels.get_unchecked(global_offset)
    }
}

fn sample_clamp(src: &[BGRA8], src_width: usize, src_height: usize, x: isize, y: isize) -> BGRA8 {
    let x = x.clamp(0, src_width as isize - 1) as usize;
    let y = y.clamp(0, src_height as isize - 1) as usize;

    src[y * src_width + x]
}

fn sample_wrap(src: &[BGRA8], src_width: usize, src_height: usize, x: isize, y: isize) -> BGRA8 {
    let x = x.rem_euclid(src_width as isize) as usize;
    let y = y.rem_euclid(src_height as isize) as usize;

    src[y * src_width + x]
}

fn downscale_3x3_box_filter(src: &[BGRA8], src_width: usize, src_height: usize, dst: &mut [BGRA8]) {
    let dst_width = src_width / 2;
    let dst_height = src_height / 2;

    assert!(dst.len() >= dst_width * dst_height);

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            let src_x = (dst_x * 2) as isize;
            let src_y = (dst_y * 2) as isize;

            // [a, b, c
            //  d, e, f
            //  g, h, i]

            let samples = [
                sample_wrap(src, src_width, src_height, src_x - 1, src_y - 1),
                sample_wrap(src, src_width, src_height, src_x, src_y - 1),
                sample_wrap(src, src_width, src_height, src_x + 1, src_y - 1),
                sample_wrap(src, src_width, src_height, src_x - 1, src_y),
                sample_wrap(src, src_width, src_height, src_x, src_y),
                sample_wrap(src, src_width, src_height, src_x + 1, src_y),
                sample_wrap(src, src_width, src_height, src_x - 1, src_y + 1),
                sample_wrap(src, src_width, src_height, src_x, src_y + 1),
                sample_wrap(src, src_width, src_height, src_x + 1, src_y + 1),
            ];

            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            let mut a = 0;

            let mut sample_count = 0;

            for sample in samples.iter() {
                if sample.a == 0 {
                    continue;
                }

                r += sample.r as u32;
                g += sample.g as u32;
                b += sample.b as u32;
                a += sample.a as u32;

                sample_count += 1;
            }

            if sample_count == 0 {
                continue;
            }

            r /= sample_count;
            g /= sample_count;
            b /= sample_count;
            a /= sample_count;

            dst[dst_y * dst_width + dst_x] = BGRA8::new(r as u8, g as u8, b as u8, a as u8);
        }
    }
}
