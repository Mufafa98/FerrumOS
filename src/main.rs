#![feature(custom_test_frameworks)]
// #![feature(abi_x86_interrupt)]
#![test_runner(ferrum_os::utils::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

use ferrum_os::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    ferrum_os::init();

    // fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

    // // trigger a stack overflow
    // stack_overflow();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    #[cfg(test)]
    test_main();

    println!("it did not crash");

    hlt_loop()
}
