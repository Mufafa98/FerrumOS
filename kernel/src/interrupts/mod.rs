//! Module for handling interrupts

use crate::gdt;
use handlers::*;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

pub mod handlers;
mod tests;

/// Programmable Interrupt Controller used for hardware
/// interrupts
///
/// The reason for the offset is so it does not
/// interfer with the CPU interrupts like DoubleFault
// pub const PIC_1_OFFSET: u8 = 32;
/// Programmable Interrupt Controller used for hardware
/// interrupts
///
/// The reason for the offset is so it does not
/// interfer with the CPU interrupts like DoubleFault
// pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
/// Static used for comunication between interrupt handler and CPU
///
/// Mainly used to notify the CPU that the interrupt has been handled
/// and can continue to send other interrupts
// pub static PICS: spin::Mutex<ChainedPics> =
//     spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// #[derive(Debug, Clone, Copy)]
// #[repr(u8)]
// /// Indexes for the different interrupts
// pub enum InterruptIndex {
//     Timer = PIC_1_OFFSET,
//     Keyboard,
// }

// impl InterruptIndex {
//     fn as_u8(self) -> u8 {
//         self as u8
//     }
//     // Unused: To be removed
//     // fn as_usize(self) -> usize {
//     //     usize::from(self.as_u8())
//     // }
// }
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndexAPIC {
    Timer = 32,
    LAPICTimer,
    Keyboard,
    HPET,
    //Test ata
    AtaMaster,
    AtaSlave,
    //Test ata
    Spurious = 0xFF,
}
impl InterruptIndexAPIC {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    pub fn as_u32(self) -> u32 {
        u32::from(self.as_u8())
    }
}

lazy_static! {
    /// Interrupt Descriptor Table used for handling interrupts
    /// and calling the correct handler
    static ref IDT: InterruptDescriptorTable = {
        // TO DO : Better implement handlers {trap gate?}
        // https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf
        // pg 201
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
        // TO DO : DOCUMENTATION
        idt.divide_error.set_handler_fn(division_error_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt[InterruptIndexAPIC::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndexAPIC::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndexAPIC::LAPICTimer.as_u8()].set_handler_fn(lapic_timer_handler);
        idt[InterruptIndexAPIC::HPET.as_u8()].set_handler_fn(hpet_interrupt_handler);
        // idt[InterruptIndexAPIC::AtaMaster.as_u8()].set_handler_fn(ata_test);
        // idt[InterruptIndexAPIC::AtaSlave.as_u8()].set_handler_fn(ata_test);
        idt
    };
}
/// Load the Interrupt Descriptor Table
pub fn init_idt() {
    IDT.load();
}
