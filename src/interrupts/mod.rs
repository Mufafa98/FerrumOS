use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

mod handlers;
mod tests;
use handlers::*;
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}
pub fn init_idt() {
    IDT.load();
}
