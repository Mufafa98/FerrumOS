//! This module contains the implementation of the PsfFont struct,
//! which is used to represent a font in the PSF format.
use super::super::framebuffer::FrameBuffer;
use glyph::Glyphs;
mod glyph;
/// PsfFont struct containing the font data
#[derive(Debug)]
#[repr(C)]
pub struct PsfFont {
    magic: u32,
    version: u32,
    headersize: u32,
    flags: u32,
    numglyph: u32,
    bytesperglyph: u32,
    height: u32,
    width: u32,
    glyphs: Glyphs,
    padding_before: u64,
    padding_after: u64,
}

impl PsfFont {
    /// Creates a new PsfFont from the given data
    ///
    /// Note: More general implementation needed
    pub fn from(data: &[u8]) -> Self {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let headersize = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let flags = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let numglyph = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let bytesperglyph = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let height = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let width = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        let glyphs = Glyphs::new(&data[32..], numglyph, bytesperglyph);
        let result = PsfFont {
            magic,
            version,
            headersize,
            flags,
            numglyph,
            bytesperglyph,
            height,
            width,
            glyphs,
            padding_before: 0, // Default padding before
            padding_after: 0,  // Default padding after
        };

        result
    }
    /// Gets the height of the font
    pub fn get_height(&self) -> u32 {
        self.height
    }
    /// Gets the width of the font
    pub fn get_width(&self) -> u32 {
        self.width + self.padding_before as u32 + self.padding_after as u32
    }
    pub fn set_padding_before(&mut self, padding: u64) {
        self.padding_before = padding;
    }
    pub fn set_padding_after(&mut self, padding: u64) {
        self.padding_after = padding;
    }
    /// Displays a character on the framebuffer at the given position with the given colors
    pub fn display_char(
        &self,
        character: char,
        framebuffer: &FrameBuffer,
        position: (u64, u64),
        fg_color: u32,
        bg_color: u32,
        font_size_multiplier: u64,
    ) {
        let glyph = self.find_glyph(character as u32).unwrap_or([0; 16]);

        let font_height = self.get_height() as u64;
        let font_width =
            self.get_width() as u64 - self.padding_before as u64 - self.padding_after as u64;

        let padding_before = self.padding_before as u64;
        let padding_after = self.padding_after as u64;

        for row in 0..font_height {
            let total_cell_width = font_width + padding_before + padding_after;
            for col in 0..total_cell_width {
                let color;

                if col >= padding_before && col < padding_before + font_width {
                    let glyph_col = col - padding_before;
                    let bit = glyph[row as usize] & (1 << (font_width - 1 - glyph_col));
                    color = {
                        if bit != 0 {
                            fg_color
                        } else {
                            bg_color
                        }
                    };
                } else {
                    color = bg_color;
                }

                let pixel_coord = (
                    position.0 + col * font_size_multiplier,
                    position.1 + row * font_size_multiplier,
                );
                // let color = if bit != 0 { fg_color } else { bg_color };
                framebuffer.put_pixel_on_square(
                    pixel_coord.0,
                    pixel_coord.1,
                    color,
                    font_size_multiplier,
                );
            }
        }
    }

    pub fn display_bold_char(
        &self,
        character: char,
        framebuffer: &FrameBuffer,
        position: (u64, u64),
        fg_color: u32,
        bg_color: u32,
        font_size_multiplier: u64,
    ) {
        // TODO: Better document
        todo!("Bold rendering is not implemented yet");
        // For bitmap fonts, a bold effect is usually achieved by drawing the character
        // twice with a 1-pixel horizontal offset.
        // Setting BOLD_OFFSET to 1 is generally sufficient.
        const BOLD_OFFSET: u64 = 1; // How many pixels to offset the second drawing

        let glyph = self.find_glyph(character as u32).unwrap_or([0; 16]);

        let font_height = self.get_height() as u64;
        let font_width = self.width as u64;

        // Draw the background for the entire cell first
        // The cell width should account for the bold offset if the font is monospace
        // and you want the cell to expand for bold characters.
        // If you always draw characters within a fixed-width cell, this part might need adjustment
        // based on how your terminal handles cell width.
        let effective_char_width = font_width + BOLD_OFFSET; // Account for the bold rendering

        for row in 0..font_height {
            for col in 0..effective_char_width {
                // Loop over the potentially expanded width
                let pixel_coord = (
                    position.0 + col * font_size_multiplier,
                    position.1 + row * font_size_multiplier,
                );
                framebuffer.put_pixel_on_square(
                    pixel_coord.0,
                    pixel_coord.1,
                    bg_color, // Draw background for the whole cell first
                    font_size_multiplier,
                );
            }
        }

        // --- Draw the character (foreground) ---

        // First pass: Draw the regular character
        for row in 0..font_height {
            for col in 0..font_width {
                let bit = glyph[row as usize] & (1 << (font_width - 1 - col));
                if bit != 0 {
                    let pixel_coord = (
                        position.0 + col * font_size_multiplier,
                        position.1 + row * font_size_multiplier,
                    );
                    framebuffer.put_pixel_on_square(
                        pixel_coord.0,
                        pixel_coord.1,
                        fg_color,
                        font_size_multiplier,
                    );
                }
            }
        }

        // Second pass: Draw the character again, offset to the right by BOLD_OFFSET
        // This creates the bold effect by thickening the strokes.
        for row in 0..font_height {
            for col in 0..font_width {
                let bit = glyph[row as usize] & (1 << (font_width - 1 - col));
                if bit != 0 {
                    // Ensure we don't draw outside the logical character cell if BOLD_OFFSET is too large.
                    // For most monospace fonts, BOLD_OFFSET of 1 is fine and often extends slightly.
                    let offset_col = col + BOLD_OFFSET;
                    // You might want to cap `offset_col` at `font_width` or `effective_char_width - 1`
                    // if you absolutely want to contain the bold effect within the original font_width.
                    // However, for true bolding, letting it spill over slightly is typical.

                    let pixel_coord = (
                        position.0 + offset_col * font_size_multiplier,
                        position.1 + row * font_size_multiplier,
                    );
                    framebuffer.put_pixel_on_square(
                        pixel_coord.0,
                        pixel_coord.1,
                        fg_color,
                        font_size_multiplier,
                    );
                }
            }
        }
    }

    /// Gets the glyph data for the given index
    fn get_glyph(&self, index: u32) -> [u8; 16] {
        let mut glyph = [0; 16];
        for i in 0..16 {
            glyph[i] = self.glyphs.get_glyph(index as usize)[i];
        }
        glyph
    }
    /// Finds the glyph for the given unicode
    fn find_glyph(&self, unicode: u32) -> Option<[u8; 16]> {
        for i in 0..self.numglyph {
            if self.glyphs.get_unicode(i as usize) == unicode {
                return Some(self.get_glyph(i));
            }
        }
        None
    }
}
