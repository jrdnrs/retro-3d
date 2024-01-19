use crate::{
    colour::BGRA8,
    font::{AlignHeight, AlignWidth, Font},
};

use super::RendererState;

pub struct TextRenderer {}

impl TextRenderer {
    pub fn new(state: &RendererState) -> Self {
        Self {}
    }

    pub fn render(
        &self,
        state: &mut RendererState,
        font: &Font,
        colour: BGRA8,
        align: (AlignWidth, AlignHeight),
        x: usize,
        y: usize,
        text: &str,
    ) {
        debug_assert!(text.is_ascii());

        let mut text_width = 0;
        let mut text_height = font.char_height();
        let mut current_line_width = 0;

        for c in text.chars() {
            if c == '\n' {
                text_height += font.char_height();
                text_width = text_width.max(current_line_width);
                current_line_width = 0;
            } else {
                current_line_width += font.char_width();
            }
        }

        let x = match align.0 {
            AlignWidth::Left => x,
            AlignWidth::Centre => x.saturating_sub(text_width / 2),
            AlignWidth::Right => x.saturating_sub(text_width),
        };

        let y = match align.1 {
            AlignHeight::Top => y,
            AlignHeight::Centre => y.saturating_sub(text_height / 2),
            AlignHeight::Bottom => y.saturating_sub(text_height),
        };

        // Exit if all text is offscreen
        if x >= state.framebuffer.width() || y >= state.framebuffer.height() {
            return;
        }

        self.draw_text(state, font, colour, x, y, text);
    }

    fn draw_text(
        &self,
        state: &mut RendererState,
        font: &Font,
        colour: BGRA8,
        x: usize,
        y: usize,
        text: &str,
    ) {
        let mut offset_x = 0;
        let mut offset_y = 0;

        for c in text.chars() {
            if c == '\n' {
                offset_x = 0;
                offset_y += font.char_height();
            } else {
                // Exit early if rest of line(s) are offscreen
                if y + offset_y >= state.framebuffer.height() {
                    return;
                }

                // Skip drawing if character is offscreen
                if x + offset_x >= state.framebuffer.width() {
                    offset_x += font.char_width();
                    continue;
                }

                self.draw_char(state, font, colour, c, x + offset_x, y + offset_y);
                offset_x += font.char_width();
            }
        }
    }

    fn draw_char(
        &self,
        state: &mut RendererState,
        font: &Font,
        colour: BGRA8,
        c: char,
        x: usize,
        y: usize,
    ) {
        // At this point, we know at least part of this character is onscreen
        debug_assert!(x < state.framebuffer.width() && y < state.framebuffer.height());

        let char_index = c as usize;
        let metadata = font.char_metadata()[char_index];

        let mut offset_x = 0;
        let mut offset_y = 0;

        for i in metadata.offset()..(metadata.offset() + metadata.len()) {
            let run_length = font.run_lengths()[i];

            if run_length.value() > 0 {
                // Clamp line length to the framebuffer width
                let len = run_length
                    .len()
                    .min(state.framebuffer.width().saturating_sub(x + offset_x));

                unsafe {
                    state
                        .framebuffer
                        .draw_h_line_unchecked(x + offset_x, y + offset_y, len, colour)
                }
            }

            offset_x += run_length.len();
            if offset_x >= font.char_width() {
                offset_x = 0;
                offset_y += 1;
            }

            // Exit early if we've reached the bottom of the framebuffer
            if y + offset_y >= state.framebuffer.height() {
                return;
            }
        }
    }
}
