#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
// TO SOLVE
#![feature(custom_test_frameworks)]
// #![test_runner(utils::test_runner)]
// #![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)]

extern crate alloc;
//maybe refactor in memory?
pub mod allocator;
//-------------------------
pub mod drivers;
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod tests;
pub mod utils;
//maybe refactor in multiTasking or sth?
pub mod task;
//--------------------------------------

// TO SOLVE
// #[cfg(test)]
// use utils::test_runner;
/// Function to initialize necessary functionalities of the kernel
/// such as gdt or interrupts
pub fn init() {
    //gdt::init();
    //interrupts::init_idt();
    //unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
/// Performant empty loop thet saves cpu time
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// TO SOLVE
// #[cfg(test)]
// use bootloader_api::{entry_point, BootInfo};

// #[cfg(test)]
// entry_point!(test_kernel_main);

// #[cfg(test)]
// fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
//     init();
//     test_main();
//     hlt_loop();
// }
