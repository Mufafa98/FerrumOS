//! This module contains the implementation of a text writer that writes to the framebuffer
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

mod bub_tw; // Bottom Up Basic Text Writer
mod tdf_tw; // Top Down Fancy Text Writer

use crate::serial_println;

use super::super::framebuffer::FRAMEBUFFER;
use super::psf_font::PsfFont;
use super::DEFAULT_FONT_DATA_BYTES;

use bub_tw::BubTextWriter;
use tdf_tw::TdfTextWriter;

lazy_static! {
    /// Text writer global instance
    pub static ref TEXT_WRITER: Mutex<TdfTextWriter> = Mutex::new(TdfTextWriter::new());
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
    // if args.as_str() == Some("\x1B[2J\x1B[1;1H") {
    //     interrupts::without_interrupts(|| {
    //         TEXT_WRITER.lock().clear_screen();
    //     });
    //     return;
    // }
    interrupts::without_interrupts(|| {
        TEXT_WRITER.lock().write_fmt(args).unwrap();
    });
}
