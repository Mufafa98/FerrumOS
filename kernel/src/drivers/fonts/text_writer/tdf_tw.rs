use super::PsfFont;
use super::DEFAULT_FONT_DATA_BYTES;
use super::FRAMEBUFFER;
use crate::drivers::fonts::color::Color;
use crate::ok;
use crate::{println, serial_print, serial_println};
use alloc::format;

use super::super::ansii_parser;

use core::fmt;

/// Text writer struct containing the current position, colors, font and font size multiplier
pub struct TdfTextWriter {
    x_position: u64,
    y_position: u64,
    // Default foreground and background colors
    fg_color: Color,
    bg_color: Color,
    /// Font used for rendering text
    font: PsfFont,
    /// Multiplier for the font size
    font_size_multiplier: u32,
    /// Saved cursor position
    saved_cursor: Option<(u64, u64)>,
    /// Modifiers
    bold: bool,
}

impl ansii_parser::Performer for TdfTextWriter {
    fn print(&mut self, c: char) {
        self.write_char(c);
    }
    fn set_bg_color(&mut self, color: u32) {
        self.bg_color = Color::from_u32(color);
    }
    fn set_fg_color(&mut self, color: u32) {
        self.fg_color = Color::from_u32(color);
    }
    fn backspace(&mut self) {
        if self.x_position > self.font.get_width() as u64 {
            self.x_position -= self.font.get_width() as u64;
            // If the x position is now less than 0, reset it to 0
        }
    }
    fn clear_screen(&mut self) {
        // Clear the framebuffer by filling it with the background color
        let screen_width = FRAMEBUFFER.get_width();
        let screen_height = FRAMEBUFFER.get_height();
        for row in 0..screen_height {
            for col in 0..screen_width {
                FRAMEBUFFER.put_pixel(col, row, self.bg_color.to_u32());
            }
        }
    }
    fn move_cursor(&mut self, row: u64, col: u64) {
        let char_width = self.font.get_width() as u64;
        let char_height = (self.font.get_height() * self.font_size_multiplier) as u64;

        // Ensure the row and column are within bounds
        let n_row = core::cmp::min(row, FRAMEBUFFER.get_height() / char_height);
        let n_col = core::cmp::min(col, FRAMEBUFFER.get_width() / char_width);

        self.x_position = n_col * char_width;
        self.y_position = n_row * char_height;
    }
    fn save_cursor(&mut self) {
        // Save the current cursor position
        let char_width = self.font.get_width() as u64;
        let char_height = (self.font.get_height() * self.font_size_multiplier) as u64;
        self.saved_cursor = Some((self.x_position / char_width, self.y_position / char_height));
    }
    fn print_cursor_position(&mut self) {
        let char_width = self.font.get_width() as u64;
        let char_height = (self.font.get_height() * self.font_size_multiplier) as u64;

        // Print the current cursor position in a format like "Row: 1, Col: 1"
        if let Some((x, y)) = self.saved_cursor {
            let row = y / char_height + 1; // Convert to 1-based index
            let col = x / char_width + 1; // Convert to 1-based index
            self.write_string(&format!("Row: {}, Col: {}\n", row, col));
        } else {
            self.write_string("Cursor position not saved.\n");
        }
    }
    fn move_cursor_to_start(&mut self) {
        // Move the cursor to the start of the screen
        self.x_position = self.font.get_width() as u64; // Start at the first character position
    }
    fn get_cursor_position(&self) -> (u64, u64) {
        // Return the current cursor position
        let x_pos = self.x_position / self.font.get_width() as u64;
        let y_pos = self.y_position / (self.font.get_height() * self.font_size_multiplier) as u64;
        (x_pos, y_pos)
    }
    fn get_saved_cursor_position(&self) -> Option<(u64, u64)> {
        // Return the saved cursor position if it exists
        self.saved_cursor
    }
    fn set_bold(&mut self, bold: bool) {
        // Set the bold modifier
        self.bold = bold;
    }
}

impl TdfTextWriter {
    /// Creates a new text writer
    pub fn new() -> Self {
        let mut font = PsfFont::from(DEFAULT_FONT_DATA_BYTES);

        font.set_padding_before(1); // No padding before
        font.set_padding_after(1); // No padding after

        let tdf = TdfTextWriter {
            x_position: font.get_width() as u64, // Start at the first character position
            y_position: font.get_height() as u64, // Start at the first line
            fg_color: Color::new(255, 255, 255, 255),
            bg_color: Color::new(0, 0, 0, 255),
            font,
            font_size_multiplier: 1,
            saved_cursor: None,
            bold: false,
        };
        tdf
    }

    /// Writes a character to the framebuffer
    pub fn write_char(&mut self, character: char) {
        let char_width = self.font.get_width() as u64;
        match character {
            '\n' => {
                self.write_newline();
                // self.print_caret();
                return;
            }
            _ => {
                // Wrap to the next line if the character would not fit.
                if self.x_position + char_width > FRAMEBUFFER.get_width() {
                    self.write_newline();
                }
                if self.bold {
                    // If bold is set, use the bold version of the font
                    self.font.display_bold_char(
                        character,
                        &FRAMEBUFFER,
                        (self.x_position, self.y_position),
                        self.fg_color.to_u32(),
                        self.bg_color.to_u32(),
                        self.font_size_multiplier as u64,
                    );
                } else {
                    // Otherwise, use the regular font
                    self.font.display_char(
                        character,
                        &FRAMEBUFFER,
                        (self.x_position, self.y_position),
                        self.fg_color.to_u32(),
                        self.bg_color.to_u32(),
                        self.font_size_multiplier as u64,
                    );
                }

                // Move the x position to the right
                self.x_position += char_width;
            }
        }
    }

    /// Writes a string to the framebuffer
    pub fn write_string(&mut self, string: &str) {
        let mut parser = ansii_parser::AnsiiParser::new(self);
        for byte in string.bytes() {
            parser.parse(byte);
        }
    }
    /// Writes a newline to the framebuffer
    /// This function moves all the rows up by the height of a character
    fn write_newline(&mut self) {
        let char_height = (self.font.get_height() * self.font_size_multiplier) as u64;
        let char_width = self.font.get_width() as u64;
        self.x_position = char_width;
        self.y_position += char_height;

        // If the cursor is off the bottom of the screen, scroll up one line.
        if self.y_position + char_height > FRAMEBUFFER.get_height() {
            self.scroll();
            // After scrolling, the cursor's y position moves up by one line height.
            self.y_position -= char_height;
        }
    }

    /// Scrolls the entire framebuffer content up by one character height.
    /// The new line at the bottom is cleared with the background color.
    fn scroll(&mut self) {
        let char_height = (self.font.get_height() * self.font_size_multiplier) as u64;
        let screen_height = FRAMEBUFFER.get_height();
        let screen_width = FRAMEBUFFER.get_width();

        // Copy every row up by char_height.
        for row in char_height..screen_height {
            for col in 0..screen_width {
                let pixel = FRAMEBUFFER.get_pixel(col, row);
                FRAMEBUFFER.put_pixel(col, row - char_height, pixel);
            }
        }

        // Clear the last line of the screen.
        let last_line_start = screen_height - char_height;
        for row in last_line_start..screen_height {
            for col in 0..screen_width {
                FRAMEBUFFER.put_pixel(col, row, self.bg_color.to_u32());
            }
        }
    }

    pub fn prin_available_chars(&mut self) {
        use alloc::string::String;
        let mut output = String::new();
        let unicodes = self.font.get_glyphs_unicodes();
        for unicode in unicodes {
            if let Some(character) = core::char::from_u32(unicode) {
                output.push(character);
            } else {
                output.push('?'); // Use '?' for invalid unicode characters
            }
        }
        self.write_string(&output);
    }
}

impl fmt::Write for TdfTextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
