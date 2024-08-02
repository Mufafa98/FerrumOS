#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ferrum_os::utils::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[allow(unused_imports)]
use core::panic::PanicInfo;
#[allow(unused_imports)]
use ferrum_os::utils::panic_module::panic;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    ferrum_os::hlt_loop();
}

use ferrum_os::println;

#[test_case]
fn test_println() {
    println!("test_println output");
}
