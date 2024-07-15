#![no_std]
#![no_main]
use drivers::*;
use io::serial_println;
#[allow(unused_imports)]
use utils::panic_module::panic;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("-----Kernel Entry Point-----");
    println!("Hello World{}", "!");
    // panic!("sall");
    kernel::init();
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };
    x86_64::instructions::interrupts::int3(); // new
    println!("It did not crash!");

    // println!("tbGARvUwPLU2XRk0ebRJMteL3wfBt1kFx6jXynk5bhF59hmjtuf9Qx4DkgaHbPyGAmztbkP3WtYxRDuCZ");

    loop {}
}
