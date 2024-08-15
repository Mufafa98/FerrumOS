#![no_std]
#![no_main]

use ferrum_os::*;

use task::{executor, keyboard, Task};

extern crate alloc;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    serial_println!("<-------------------->\n FerrumOs has started\n<-------------------->");
    ferrum_os::init();
    // loop {
    //     print!("-");
    // }
    println!("Hello World from println macro!");
    println!("Hello World from Again macro!");
    let mut executor = executor::Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
    // hlt_loop();
}
#[allow(dead_code)]
fn heap_test_debug() {
    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
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
