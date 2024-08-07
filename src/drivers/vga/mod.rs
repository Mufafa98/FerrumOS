//! Offers support for operations on the VGA memory

use colors::*;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::*;

mod colors;
mod tests;

lazy_static! {
    /// A mutex protected static used for writing to the screen.
    ///
    /// It's main usage is in the print! and println! macros
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Prints to the STOUT trough the VGA interface
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}
/// Prints to the STOUT trough the VGA interface, appending a newline
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
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
/// Represents a symbol that can be printed using
/// the VGA buffer.
///
/// It has two fields thet represent the symbol's
/// code and it's colors(foreground & background)
pub struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}
impl ScreenChar {
    /// Returns the character stored in the ScreenChar
    /// struct
    pub fn get_ascii_char(&self) -> char {
        char::from(self.ascii_char)
    }
}
/// Max number of lines that can be printed
pub const BUFFER_HEIGHT: usize = 25;
/// Max number of symbols per line that can be printed
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
/// Memory representation of what is shown on the screen
/// as a grid of BUFFER_HEIGHT x BUFFER_WIDTH
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
impl Buffer {
    /// Retrives the character possitioned on
    /// the row and column provided
    pub fn get_char(&self, row: usize, col: usize) -> &Volatile<ScreenChar> {
        &self.chars[row][col]
    }
}
/// Responsable for writing on the screen.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Returns the current stored buffer
    ///
    /// Mainly used for tests, may be removed in
    /// future versions
    pub fn get_buffer(&mut self) -> &Buffer {
        self.buffer
    }
    /// Writes a string into the buffer using write_byte()
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
    /// Writes a byte into the buffer. The writing direction
    /// is Left to Right and Bottom to Top
    ///
    /// Note that the only characters suported are
    /// those present in Code page 437. Therefore
    /// any other character that is not present in
    /// this list will be replaced by the character
    /// 0xfe
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    /// Moves all the rows up by one and clear the
    /// bottom most row
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    /// Clears a specified row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}
