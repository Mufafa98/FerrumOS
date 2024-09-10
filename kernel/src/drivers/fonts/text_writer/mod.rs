//! This module contains the implementation of a text writer that writes to the framebuffer
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::serial_println;

use super::super::framebuffer::FRAMEBUFFER;
use super::psf_font::PsfFont;
use super::DEFAULT_FONT_DATA_BYTES;

/// Text writer struct containing the current position, colors, font and font size multiplier
pub struct TextWriter {
    x_position: u64,
    fg_color: Color,
    bg_color: Color,
    font: PsfFont,
    font_size_multiplier: u32,
}
impl TextWriter {
    /// Creates a new text writer
    pub fn new() -> Self {
        let font = PsfFont::from(DEFAULT_FONT_DATA_BYTES);

        TextWriter {
            x_position: 0,
            fg_color: Color::new(255, 255, 255, 255),
            bg_color: Color::new(0, 122, 1, 1),
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
    // TO DO : Optimize this function
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
    // TO DO : Optimize this function
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
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for character in s.chars() {
            self.write_char(character);
        }
        Ok(())
    }
}
/// Color struct containing the red, green, blue and alpha values
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Creates a new color with the given red, green, blue and alpha values
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    /// Converts the color to a u32 value
    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

lazy_static! {
    /// Text writer global instance
    pub static ref TEXT_WRITER: Mutex<TextWriter> = Mutex::new(TextWriter::new());
}

/// Prints to the STOUT trough the framebuffer interface
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::fonts::text_writer::_print(format_args!($($arg)*)));
}
/// Prints to the STOUT trough the framebuffer interface, appending a newline

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    //disable interrupts while printing a message
    interrupts::without_interrupts(|| {
        TEXT_WRITER.lock().write_fmt(args).unwrap();
    });
}
