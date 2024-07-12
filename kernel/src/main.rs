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

    // println!("tbGARvUwPLU2XRk0ebRJMteL3wfBt1kFx6jXynk5bhF59hmjtuf9Qx4DkgaHbPyGAmztbkP3WtYxRDuCZ");

    loop {}
}
