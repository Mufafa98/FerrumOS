//use drivers::{println, vga::*};
#[cfg(test)]
use crate::drivers::vga::{BUFFER_HEIGHT, WRITER};
#[cfg(test)]
use crate::println;

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER
            .lock()
            .get_buffer()
            .get_char(BUFFER_HEIGHT - 2, i)
            .read();
        assert_eq!(screen_char.get_ascii_char(), c);
    }
}
