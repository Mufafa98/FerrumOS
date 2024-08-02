#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
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

#[cfg(test)]
use utils::test_runner;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}
