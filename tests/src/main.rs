#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner_function)]
#![reexport_test_harness_main = "test_main"]

mod drivers_;
mod kernel_;

use io::serial_println;
#[allow(unused_imports)]
use utils::panic_module::panic;
use utils::test_runner_function;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("-----Tests Entry Point-----");

    #[cfg(test)]
    test_main();

    loop {}
}
