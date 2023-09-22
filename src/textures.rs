use std::{fs::File, path::Path};

use crate::colour::BGRA8;

/// The number of mip levels to generate for each texture, where level 0 is the original size and
/// subsequent levels are half the size of the previous level
pub const MIP_LEVELS: usize = 4;
/// Arbitrary factor to scale the mip level distance thresholds by. A higher value will result in
/// more mip levels being used for a given distance
pub const MIP_FACTOR: f32 = 8.0;
/// As subsequent mip maps are smaller resolutions, we use this to scale texture coordinates
pub const MIP_SCALES: [f32; MIP_LEVELS] = [1.0 / 1.0, 1.0 / 2.0, 1.0 / 4.0, 1.0 / 8.0];

pub const PLACEHOLDER: usize = 0;
pub const BRICK: usize = 1;
pub const ROCK: usize = 2;
pub const STONE: usize = 3;
pub const STONE_BRICK: usize = 4;
pub const PLANK: usize = 5;
pub const GRASS: usize = 6;
pub const DIRT: usize = 7;
pub const SAND: usize = 8;
pub const CONCRETE: usize = 9;
pub const LEAF: usize = 10;
pub const OBSIDIAN: usize = 11;
pub const PORTAL: usize = 12;

#[derive(Debug)]
struct Bitmap {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

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
    pub fn sample(&self, x: usize, y: usize, level: usize) -> BGRA8 {
        let local_offset = y * self.levels[level].width + x;
        let global_offset = self.levels[level].offset + local_offset;

        self.pixels[global_offset]
    }

    fn from_bitmap(bitmap: Bitmap) -> Self {
        let levels = Self::calculate_mip_levels(&bitmap);
        let buffer_size = levels[MIP_LEVELS - 1].offset
            + levels[MIP_LEVELS - 1].width * levels[MIP_LEVELS - 1].height;

        let mut pixels = vec![BGRA8::default(); buffer_size];

        // Copy the pixels from the bitmap into the first level of the texture
        unsafe {
            core::ptr::copy_nonoverlapping(
                bitmap.pixels.as_ptr(),
                pixels.as_mut_ptr() as *mut u8,
                bitmap.pixels.len(),
            );
        }

        Self::generate_mip_maps(&levels, &mut pixels);

        Self { levels, pixels }
    }

    fn calculate_mip_levels(bitmap: &Bitmap) -> [MipLevel; MIP_LEVELS] {
        let mut mip_width = bitmap.width;
        let mut mip_height = bitmap.height;
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
}

#[derive(Debug)]
pub struct Textures {
    textures: Vec<Texture>,
}

impl Textures {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Texture> {
        self.textures.get(index)
    }

    pub fn load_default(&mut self) {
        let texture_files = [
            "./assets/textures/placeholder.png",
            "./assets/textures/brick.png",
            "./assets/textures/rock.png",
            "./assets/textures/stone.png",
            "./assets/textures/stone_brick.png",
            "./assets/textures/plank.png",
            "./assets/textures/grass.png",
            "./assets/textures/dirt.png",
            "./assets/textures/sand.png",
            "./assets/textures/concrete.png",
            "./assets/textures/leaf.png",
            "./assets/textures/obsidian.png",
            "./assets/textures/portal.png",
        ];

        for file in texture_files {
            self.load_png(file).unwrap();
        }
    }

    fn load_png(&mut self, path: impl AsRef<Path>) -> Result<usize, &'static str> {
        let mut decoder = png::Decoder::new(File::open(path).map_err(|_| "Failed to open file")?);
        decoder.set_transformations(png::Transformations::ALPHA | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().map_err(|_| "Failed to read info")?;
        let mut buffer = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buffer)
            .map_err(|_| "Failed to read frame")?;

        assert_eq!(
            info.bit_depth,
            png::BitDepth::Eight,
            "Unsupported bit depth"
        );

        assert_eq!(
            info.color_type,
            png::ColorType::Rgba,
            "Unsupported colour type"
        );

        // Convert from RGBA to BGRA
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0,2);
        }

        // match info.color_type {
        //     png::ColorType::Rgba => {
        //         // Strip alpha channel
        //         let mut i = 0;
        //         let mut j = 0;

        //         while i < buffer.len() {
        //             buffer[j] = buffer[i];
        //             buffer[j + 1] = buffer[i + 1];
        //             buffer[j + 2] = buffer[i + 2];

        //             i += 4;
        //             j += 3;
        //         }

        //         buffer.resize(j, 0);
        //     }

        //     png::ColorType::Rgb => {}

        //     _ => return Err("Unsupported colour type"),
        // }

        let bitmap = Bitmap {
            width: info.width as usize,
            height: info.height as usize,
            pixels: buffer,
        };

        let texture = Texture::from_bitmap(bitmap);

        self.textures.push(texture);

        Ok(self.textures.len() - 1)
    }
}

fn sample_clamp(src: &[BGRA8], src_width: usize, src_height: usize, x: usize, y: usize) -> BGRA8 {
    let x = x.min(src_width - 1);
    let y = y.min(src_height - 1);

    src[y * src_width + x]
}

fn sample_wrap(src: &[BGRA8], src_width: usize, src_height: usize, x: usize, y: usize) -> BGRA8 {
    let x = x % src_width;
    let y = y % src_height;

    src[y * src_width + x]
}

fn downscale_3x3_box_filter(src: &[BGRA8], src_width: usize, src_height: usize, dst: &mut [BGRA8]) {
    let dst_width = src_width / 2;
    let dst_height = src_height / 2;

    assert!(dst.len() >= dst_width * dst_height);

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            let src_x = dst_x * 2;
            let src_y = dst_y * 2;

            // [a, b, c
            //  d, e, f
            //  g, h, i]

            let samples = [
                sample_clamp(
                    src,
                    src_width,
                    src_height,
                    src_x.saturating_sub(1),
                    src_y.saturating_sub(1),
                ),
                sample_clamp(src, src_width, src_height, src_x, src_y.saturating_sub(1)),
                sample_clamp(
                    src,
                    src_width,
                    src_height,
                    src_x + 1,
                    src_y.saturating_sub(1),
                ),
                sample_clamp(src, src_width, src_height, src_x.saturating_sub(1), src_y),
                sample_clamp(src, src_width, src_height, src_x, src_y),
                sample_clamp(src, src_width, src_height, src_x + 1, src_y),
                sample_clamp(
                    src,
                    src_width,
                    src_height,
                    src_x.saturating_sub(1),
                    src_y + 1,
                ),
                sample_clamp(src, src_width, src_height, src_x, src_y + 1),
                sample_clamp(src, src_width, src_height, src_x + 1, src_y + 1),
            ];

            let mut r = 0;
            let mut g = 0;
            let mut b = 0;

            for sample in samples.iter() {
                r += sample.r as u32;
                g += sample.g as u32;
                b += sample.b as u32;
            }

            r /= 9;
            g /= 9;
            b /= 9;

            dst[dst_y * dst_width + dst_x] = BGRA8::new(r as u8, g as u8, b as u8, 0xFF)
        }
    }
}
