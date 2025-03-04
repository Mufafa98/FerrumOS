#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

extern crate alloc;
pub mod allocator;
//-------------------------
pub mod drivers;
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod utils;
//maybe refactor in multiTasking or sth?
pub mod task;
pub mod timer;
// use lazy_static::lazy_static;
//--------------------------------------
use limine::{
    request::{HhdmRequest, KernelAddressRequest},
    BaseRevision,
};
#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();
#[used]
#[link_section = ".requests"]
static KERNEL_ADDRESS_REQUEST: KernelAddressRequest = KernelAddressRequest::new();
#[used]
#[link_section = ".requests"]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
// TO SOLVE
// #[cfg(test)]
// use utils::test_runner;
/// Function to initialize necessary functionalities of the kernel
/// such as gdt or interrupts
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    // unsafe {
    //     interrupts::PICS.lock().initialize();
    //     // TO DO : Type to faciliatte creation of the mask
    //     // here we are masking the first 2 interrupts
    //     // wich are the timer and the keyboard
    //     interrupts::PICS.lock().write_masks(0b11111100, 255);
    // };
    // unsafe { interrupts::PICS.lock().disable() };
    x86_64::instructions::interrupts::enable();
    // Mem init
    use memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;

    let phys_base = HHDM_REQUEST.get_response().unwrap().offset();
    let phys_mem_offset = VirtAddr::new(phys_base);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init() };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    use drivers::apic::{io_apic, local_apic};
    local_apic::init();
    io_apic::init();
}
/// Performant empty loop thet saves cpu time
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// #[cfg(test)]
// #[no_mangle]
// unsafe extern "C" fn _start() -> ! {
//     // init();
//     test_main();
//     // hlt_loop();
//     loop {}
// }
// #[test_case]
// fn trivial_assertion() {
//     assert_eq!(1, 1);
// }
