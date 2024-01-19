use std::path::Path;

use crate::bitmap::Bitmap;

pub enum AlignWidth {
    Left,
    Centre,
    Right,
}

pub enum AlignHeight {
    Top,
    Centre,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub struct CharMetadata {
    offset: usize,
    len: usize,
}

impl CharMetadata {
    pub fn new(offset: usize, len: usize) -> Self {
        Self { offset, len }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RunLength {
    value: u8,
    len: usize,
}

impl RunLength {
    pub fn new(value: u8, len: usize) -> Self {
        Self { value, len }
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct Font {
    char_width: usize,
    char_height: usize,
    char_metadata: Vec<CharMetadata>,
    run_lengths: Vec<RunLength>,
}

impl Font {
    pub fn from_path_png(
        path: impl AsRef<Path>,
        char_width: usize,
        char_height: usize,
        char_spacing: usize,
    ) -> Result<Self, &'static str> {
        let bitmap = Bitmap::from_path_png(path)?;

        Ok(Self::from_bitmap(
            &bitmap,
            char_width,
            char_height,
            char_spacing,
        ))
    }

    fn from_bitmap(
        bitmap: &Bitmap,
        char_width: usize,
        char_height: usize,
        char_spacing: usize,
    ) -> Self {
        let (char_metadata, run_lengths) =
            Self::encode_bitmap_font(bitmap, char_width, char_height, char_spacing);

        Self {
            char_width,
            char_height,
            char_metadata,
            run_lengths,
        }
    }

    pub fn char_width(&self) -> usize {
        self.char_width
    }

    pub fn char_height(&self) -> usize {
        self.char_height
    }

    pub fn char_metadata(&self) -> &[CharMetadata] {
        &self.char_metadata
    }

    pub fn run_lengths(&self) -> &[RunLength] {
        &self.run_lengths
    }

    fn encode_bitmap_font(
        bitmap: &Bitmap,
        char_width: usize,
        char_height: usize,
        char_spacing: usize,
    ) -> (Vec<CharMetadata>, Vec<RunLength>) {
        assert_eq!(
            (bitmap.width() + char_spacing) % (char_width + char_spacing),
            0,
            "Invalid char width or spacing"
        );
        assert_eq!(
            (bitmap.height() + char_spacing) % (char_height + char_spacing),
            0,
            "Invalid char height or spacing"
        );

        let mut run_lengths = Vec::new();
        let mut metadata = Vec::new();

        let chars_x = (bitmap.width() + char_spacing) / (char_width + char_spacing);
        let chars_y = (bitmap.height() + char_spacing) / (char_height + char_spacing);

        let mut char_run_length_offset = 0;

        for char_y in 0..chars_y {
            for char_x in 0..chars_x {
                for y in 0..char_height {
                    // We reset the run length for each row
                    let mut run_value = 0;
                    let mut run_length = 0;

                    for x in 0..char_width {
                        let pixel_x = char_x * (char_width + char_spacing) + x;
                        let pixel_y = char_y * (char_height + char_spacing) + y;

                        let pixel = bitmap.get_pixel(pixel_x, pixel_y);

                        // For now the value will be binary, where black is 0
                        let value = if pixel.r.saturating_add(pixel.g).saturating_add(pixel.b) == 0 {
                            0
                        } else {
                            1
                        };

                        if value == run_value {
                            run_length += 1;
                        } else {
                            run_lengths.push(RunLength::new(run_value, run_length));

                            run_value = value;
                            run_length = 1;
                        }
                    }

                    // Close the final run
                    run_lengths.push(RunLength::new(run_value, run_length));
                }

                // For each char, store the offset and length of its run lengths
                metadata.push(CharMetadata {
                    offset: char_run_length_offset,
                    len: run_lengths.len() - char_run_length_offset,
                });
                char_run_length_offset = run_lengths.len();
            }
        }

        (metadata, run_lengths)
    }
}
