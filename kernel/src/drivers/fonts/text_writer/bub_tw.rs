use super::PsfFont;
use super::DEFAULT_FONT_DATA_BYTES;
use super::FRAMEBUFFER;
use crate::drivers::fonts::color::Color;

use core::fmt;

/// Text writer struct containing the current position, colors, font and font size multiplier
pub struct BubTextWriter {
    x_position: u64,
    fg_color: Color,
    bg_color: Color,
    font: PsfFont,
    font_size_multiplier: u32,
}

impl BubTextWriter {
    /// Creates a new text writer
    pub fn new() -> Self {
        let font = PsfFont::from(DEFAULT_FONT_DATA_BYTES);

        BubTextWriter {
            x_position: 0,
            fg_color: Color::new(255, 255, 255, 255),
            // bg_color: Color::new(0, 122, 1, 1),
            bg_color: Color::new(0, 0, 0, 255),
            font,
            font_size_multiplier: 1,
        }
    }
    /// Writes a character to the framebuffer
    pub fn write_char(&mut self, character: char) {
        match character {
            '\n' => {
                self.write_newline();
                return;
            }
            '\x08' => {
                // Move the cursor back by one character
                self.move_back_cursor();
                return;
            }
            normal_char => {
                // Check if we reached the end of the screen
                if self.x_position > FRAMEBUFFER.get_width() {
                    self.write_newline();
                }
                // Set the row at the bottom of the screen
                let row = FRAMEBUFFER.get_height()
                    - (self.font.get_height() * self.font_size_multiplier) as u64;
                // Set the column at the current x position
                let col = self.x_position;
                self.font.display_char(
                    normal_char,
                    &FRAMEBUFFER,
                    (col, row),
                    self.fg_color.to_u32(),
                    self.bg_color.to_u32(),
                    self.font_size_multiplier as u64,
                );
                // Move the x position to the right
                self.x_position += (self.font.get_width() * self.font_size_multiplier) as u64;
            }
        }
    }
    /// Writes a string to the framebuffer
    pub fn write_string(&mut self, string: &str) {
        for character in string.chars() {
            self.write_char(character);
        }
    }
    /// Writes a newline to the framebuffer
    /// This function moves all the rows up by the height of a character
    fn write_newline(&mut self) {
        // Get the height of a character
        let char_height = (self.font.get_height() * self.font_size_multiplier) as u64;
        for row in char_height..FRAMEBUFFER.get_height() {
            for col in 0..FRAMEBUFFER.get_width() {
                // Move all the rows up by the height of a character
                let pixel = FRAMEBUFFER.get_pixel(col, row);
                if pixel == self.fg_color.to_u32() || pixel == self.bg_color.to_u32() {
                    FRAMEBUFFER.put_pixel(col, row - char_height, pixel);
                }
            }
        }
        self.clear_row(FRAMEBUFFER.get_height() - char_height);
        self.x_position = 0;
    }
    /// Clears a row of the framebuffer
    fn clear_row(&mut self, row_start: u64) {
        for row in row_start..FRAMEBUFFER.get_height() {
            for col in 0..FRAMEBUFFER.get_width() {
                // let nrow = row * FRAMEBUFFER.get_pitch();
                // let ncol = col * 4;
                let pixel = FRAMEBUFFER.get_pixel(col, row);
                if pixel == self.fg_color.to_u32() || pixel == self.bg_color.to_u32() {
                    FRAMEBUFFER.put_pixel(col, row, self.bg_color.to_u32());
                }
            }
        }
    }

    fn clear_screen(&mut self) {
        for row in 0..FRAMEBUFFER.get_height() {
            for col in 0..FRAMEBUFFER.get_width() {
                FRAMEBUFFER.put_pixel(col, row, self.bg_color.to_u32());
            }
        }
        self.x_position = 0;
    }

    fn move_back_cursor(&mut self) {
        if self.x_position > 0 {
            self.x_position -= (self.font.get_width() * self.font_size_multiplier) as u64;
        }
    }
}

impl fmt::Write for BubTextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for character in s.chars() {
            self.write_char(character);
        }
        Ok(())
    }
}
