//! Interrupt handlers for the different interrupts
use core::{ops::Add, sync::atomic::AtomicU64};

use crate::*;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode, SelectorErrorCode};
/// Handler for the breakpoint interrupt
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
/// Handler for the double fault interrupt
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
/// Handler for the page fault interrupt
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}
// TO DO: DOCUMENTATION
// TO DO: Better implementation
pub extern "x86-interrupt" fn division_error_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVISION ERROR\n{:#?}", stack_frame);
    // TO DO : Throw sth when division error to prevent infinite loop
    hlt_loop();
}
pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: INVALID TSS");
    println!(
        "Index: {:?}",
        SelectorErrorCode::new_truncate(error_code).index()
    );
    println!("{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SEGMENT NOT PRESENT");
    println!(
        "Index: {:?}",
        SelectorErrorCode::new_truncate(error_code).index()
    );
    println!("{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: STACK SEGMENT FAULT");
    println!(
        "Index: {:?}",
        SelectorErrorCode::new_truncate(error_code).index()
    );
    println!("{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT");
    println!(
        "Index: {:?}",
        SelectorErrorCode::new_truncate(error_code).index()
    );
    println!("{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}
pub static COUNTER: AtomicU64 = AtomicU64::new(0);
/// Handler for the timer interrupt
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    let ptr = COUNTER.as_ptr();
    unsafe { *ptr = COUNTER.load(core::sync::atomic::Ordering::Relaxed) + 1 };
    // TO DO: Implement EOI for LAPIC
    let mut ptr = (0xfee00000 as u32 + 0xB0 as u32) as *mut u32;
    unsafe {
        *ptr = 0;
    }
    // Notify the CPU that the interrupt has been handled
    // and can continue to send other interrupts
    // unsafe {
    //     PICS.lock()
    //         .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    // }
}
// /// Handler for the keyboard interrupt
// pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
//     use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
//     use spin::Mutex;
//     use x86_64::instructions::port::Port;

//     lazy_static! {
//         /// Keyboard instance used for handling keyboard interrupts
//         static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
//             Mutex::new(Keyboard::new(
//                 ScancodeSet1::new(),
//                 layouts::Us104Key,
//                 HandleControl::Ignore
//             ));
//     }

//     let mut _keyboard = KEYBOARD.lock();

//     let mut port = Port::new(0x60);
//     // Read the scancode from the keyboard
//     let scancode: u8 = unsafe { port.read() };
//     crate::task::keyboard::add_scancode(scancode);

//     // Notify the CPU that the interrupt has been handled
//     // and can continue to send other interrupts
//     unsafe {
//         PICS.lock()
//             .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
//     }
// }
