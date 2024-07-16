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
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });

    // old test => TO BE REMOVED
    // let s = "Some test string that fits on a single line";
    // println!("{}", s);
    // for (i, c) in s.chars().enumerate() {
    //     let screen_char = WRITER
    //         .lock()
    //         .get_buffer()
    //         .get_char(BUFFER_HEIGHT - 2, i)
    //         .read();
    //     assert_eq!(screen_char.get_ascii_char(), c);
    // }
}
