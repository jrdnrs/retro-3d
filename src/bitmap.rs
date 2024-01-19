use std::{fs::File, mem::ManuallyDrop, path::Path};

use crate::colour::BGRA8;

#[derive(Debug)]
pub struct Bitmap {
    width: usize,
    height: usize,
    pixels: Vec<BGRA8>,
}

impl Bitmap {
    pub fn new(width: usize, height: usize, pixels: Vec<BGRA8>) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn from_path_png(path: impl AsRef<Path>) -> Result<Self, &'static str> {
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
            pixel.swap(0, 2);
        }

        // Set pixel buffer type
        let len = info.width as usize * info.height as usize;
        let buffer = ManuallyDrop::new(buffer);
        let buffer = unsafe { Vec::from_raw_parts(buffer.as_ptr() as *mut BGRA8, len, len) };

        Ok(Self::new(info.width as usize, info.height as usize, buffer))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[BGRA8] {
        &self.pixels
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> BGRA8 {
        debug_assert!(x < self.width && y < self.height);

        self.pixels[y * self.width + x]
    }

    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> BGRA8 {
        debug_assert!(x < self.width && y < self.height);

        *self.pixels.get_unchecked(y * self.width + x)
    }
}
