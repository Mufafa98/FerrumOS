//! Interrupt handlers for the different interrupts
use core::sync::atomic::{AtomicBool, AtomicU64};

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
use crate::drivers::apic::local_apic::LOCAL_APIC;
use core::sync::atomic::*;
pub static PIT_COUNTER: AtomicU64 = AtomicU64::new(0);
pub static PIT_SLEEP_COUNTER: AtomicI64 = AtomicI64::new(0);
pub static PIT_SLEEP_FLAG: AtomicBool = AtomicBool::new(false);
/// Handler for the timer interrupt
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    PIT_COUNTER.store(PIT_COUNTER.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    if PIT_SLEEP_FLAG.load(Ordering::Relaxed) {
        let ptr = PIT_SLEEP_COUNTER.as_ptr();
        unsafe { *ptr = PIT_SLEEP_COUNTER.load(Ordering::Relaxed) - 1 };
    }
    // Notify the CPU that the interrupt has been handled
    // and can continue to send other interrupts
    LOCAL_APIC.set_eoi();
}
pub static LAPIC_TIMER_SLEEP_FLAG: AtomicBool = AtomicBool::new(false);
pub static LAPIC_TIMER_SLEEP_COUNTER: AtomicU64 = AtomicU64::new(0);
pub extern "x86-interrupt" fn lapic_timer_handler_old(_stack_frame: InterruptStackFrame) {
    if LAPIC_TIMER_SLEEP_FLAG.load(Ordering::Relaxed) {
        let ptr = LAPIC_TIMER_SLEEP_COUNTER.as_ptr();
        LAPIC_TIMER_SLEEP_COUNTER.fetch_sub(1, Ordering::Relaxed);
    }

    LOCAL_APIC.set_eoi();
}

//

//
pub struct SleepEntry {
    pub remaining: u64,
    pub waker: core::task::Waker,
}
use alloc::collections::BTreeMap;
use spin::Mutex;
lazy_static! {
    pub static ref SLEEP_TASKS: Mutex<BTreeMap<u64, SleepEntry>> = Mutex::new(BTreeMap::new());
}
use alloc::vec;
pub extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    {
        let mut tasks = SLEEP_TASKS.lock();
        // Collect finished tasks so we can remove them outside the iteration
        let mut finished = vec![];
        for (task_id, entry) in tasks.iter_mut() {
            if entry.remaining > 0 {
                entry.remaining -= 1;
                if entry.remaining == 0 {
                    // Wake the task
                    entry.waker.wake_by_ref();
                    finished.push(*task_id);
                }
            }
        }
        // Remove finished tasks
        for task_id in finished {
            tasks.remove(&task_id);
        }
    }

    LOCAL_APIC.set_eoi();
}
pub static HPET_SLEEP_COUNTER: AtomicI64 = AtomicI64::new(0);
pub static HPET_SLEEP_FLAG: AtomicBool = AtomicBool::new(false);
pub extern "x86-interrupt" fn hpet_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // serial_println!("HPET: {}", HPET_SLEEP_COUNTER.load(Ordering::Relaxed));
    // panic!("FIX like lapic timer");
    if HPET_SLEEP_FLAG.load(Ordering::Relaxed) {
        HPET_SLEEP_COUNTER.fetch_add(
            HPET_SLEEP_COUNTER.load(Ordering::Relaxed) + 1,
            Ordering::Relaxed,
        );
    }
    LOCAL_APIC.set_eoi();
}
/// Handler for the keyboard interrupt
pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;
    lazy_static! {
        /// Keyboard instance used for handling keyboard interrupts
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut _keyboard = KEYBOARD.lock();

    let mut port = Port::new(0x60);
    // Read the scancode from the keyboard
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    // Notify the CPU that the interrupt has been handled
    // and can continue to send other interrupts
    LOCAL_APIC.set_eoi();
}
