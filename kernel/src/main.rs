#![no_std]
#![no_main]

use core::slice;

use ferrum_os::*;

use memory::Entries;

use lazy_static::lazy_static;
use spin::mutex::Mutex;
use x86_64::structures::paging::Translate;

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial_println!("<-------------------->\n FerrumOs has started\n<-------------------->");
    ferrum_os::init();
    use ferrum_os::drivers::fonts::text_writer::*;
    let mut text_writer = TextWriter::new();
    text_writer.write_string("Hello World!\n");

    let mut mutex_writer: Mutex<TextWriter> = Mutex::new(TextWriter::new());
    mutex_writer
        .lock()
        .write_string("Hello World from Mutex!\n");
    println!("Hello World from println macro!");
    x86_64::instructions::interrupts::int3();
    hlt_loop();
}

fn heap_test_debug() {
    let heap_value = Box::new(41);
    serial_println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..5000 {
        vec.push(i);
    }
    serial_println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    serial_println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    serial_println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );
}
