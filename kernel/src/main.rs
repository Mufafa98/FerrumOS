#![no_std]
#![no_main]

use alloc::{
    fmt::format,
    format,
    string::{String, ToString},
};
use ferrum_os::*;

use task::{executor, keyboard, Task};

extern crate alloc;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    ferrum_os::init();
    let title = "FerrumOs has started";
    let separator = "-".repeat(title.len());
    let mut features = "".to_string();
    #[cfg(feature = "test")]
    features.push_str("\n Test");
    #[cfg(not(feature = "test"))]
    features.push_str("\n Default");
    serial_println!(
        "<{separator}>\n {} \n [Features]:{} \n<{separator}>",
        title,
        features,
        separator = "-".repeat(title.len())
    );
    let mut executor = executor::Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}
// #[test_case]
// fn trivial_assertion() {
//     assert_eq!(1, 1);
// }
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
// TO DO : Throw error when stack overflow
fn inf_rec() {
    inf_rec();
    x86_64::instructions::hlt();
}
