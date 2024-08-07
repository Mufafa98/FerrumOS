#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ferrum_os::utils::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    test_main();
    ferrum_os::hlt_loop();
}

use ferrum_os::println;

#[test_case]
fn test_println() {
    println!("test_println output");
}
