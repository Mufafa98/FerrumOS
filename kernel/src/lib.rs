#![no_std]
use interrupts;

pub fn init() {
    interrupts::init_idt();
}
