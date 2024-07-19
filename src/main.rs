#![feature(custom_test_frameworks)]
#![test_runner(ferrum_os::utils::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use ferrum_os::*;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    ferrum_os::init();

    use ferrum_os::memory::BootInfoFrameAllocator;
    use x86_64::structures::paging::Translate;
    use x86_64::{structures::paging::Page, VirtAddr};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = memory::EmptyFrameAllocator;
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // map an unused page
    // let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("it did not crash");

    hlt_loop()
}
