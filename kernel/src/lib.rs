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
use limine::{
    memory_map::Entry,
    request::{HhdmRequest, KernelAddressRequest, MemoryMapRequest},
    BaseRevision,
};
static BASE_REVISION: BaseRevision = BaseRevision::new();
static KERNEL_ADDRESS_REQUEST: KernelAddressRequest = KernelAddressRequest::new();
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
// TO SOLVE
// #[cfg(test)]
// use utils::test_runner;
/// Function to initialize necessary functionalities of the kernel
/// such as gdt or interrupts
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    // Mem init
    use memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;
    let phys_base = HHDM_REQUEST.get_response().unwrap().offset();
    let phys_mem_offset = VirtAddr::new(phys_base);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init() };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
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
