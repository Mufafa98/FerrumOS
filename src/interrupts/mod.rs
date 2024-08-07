//! Module for handling interrupts

use crate::gdt;
use handlers::*;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::InterruptDescriptorTable;

mod handlers;
mod tests;

/// Programmable Interrupt Controller used for hardware
/// interrupts
///
/// The reason for the offset is so it does not
/// interfer with the CPU interrupts like DoubleFault
pub const PIC_1_OFFSET: u8 = 32;
/// Programmable Interrupt Controller used for hardware
/// interrupts
///
/// The reason for the offset is so it does not
/// interfer with the CPU interrupts like DoubleFault
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
/// Static used for comunication between interrupt handler and CPU
///
/// Mainly used to notify the CPU that the interrupt has been handled
/// and can continue to send other interrupts
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// Indexes for the different interrupts
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    // Unused: To be removed
    // fn as_usize(self) -> usize {
    //     usize::from(self.as_u8())
    // }
}

lazy_static! {
    /// Interrupt Descriptor Table used for handling interrupts
    /// and calling the correct handler
    static ref IDT: InterruptDescriptorTable = {
        // Create a new IDT
        let mut idt = InterruptDescriptorTable::new();
        // Set the handler for the breakpoint interrupt
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // Set the handler for the double fault interrupt
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // Set the handler for the page fault interrupt
        idt.page_fault.set_handler_fn(page_fault_handler);
        // Set the handler for the timer interrupt
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        // Set the handler for the keyboard interrupt
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}
/// Load the Interrupt Descriptor Table
pub fn init_idt() {
    IDT.load();
}
