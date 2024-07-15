#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

pub mod drivers;
pub mod interrupts;
pub mod io;
pub mod tests;
pub mod utils;

#[cfg(test)]
use utils::test_runner;

//use utils::panic_module::panic;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");
    // panic!("asd");
    #[cfg(test)]
    test_main();
    loop {}
}
