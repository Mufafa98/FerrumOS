#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
// #![feature(const_mut_refs)]

extern crate alloc;
pub mod allocator;
pub mod drivers;
pub mod fs;
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod shell;
pub mod task;
pub mod timer;
pub mod utils;
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

/// Function to initialize necessary functionalities of the kernel
/// such as gdt or interrupts
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    x86_64::instructions::interrupts::enable();

    use memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;
    // Get the physical base address of the kernel from the bootloader
    let phys_base = HHDM_REQUEST.get_response().unwrap().offset();
    // Create a virtual address from the physical base address
    // VirtAddr is a newtype wrapper around u64
    let phys_mem_offset = VirtAddr::new(phys_base);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init() };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let kernel_address = KERNEL_ADDRESS_REQUEST.get_response().expect("nu merge");
    // println!("Kernel phys address: {:#X}", kernel_address.physical_base());
    serial_println!("Kernel virt address: {:#X}", kernel_address.virtual_base());
    println!("Kernel virt address: {:#X}", kernel_address.virtual_base());

    drivers::apic::local_apic::init();
    drivers::apic::io_apic::init();
    crate::timer::lapic::lapic_calibrate();
    drivers::ata::init();
}

/// Performant empty loop thet saves cpu time
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
