#![no_std]
#![no_main]
use core::panic::PanicInfo;

use drivers::*;

// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
