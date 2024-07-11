#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner_function)]
#![reexport_test_harness_main = "test_main"]

mod tests;

use drivers::*;
use io::serial_println;
#[allow(unused_imports)]
use utils::panic_module::panic;
#[cfg(test)]
use utils::test_runner_function;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("-----Kernel Entry Point-----");
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}
