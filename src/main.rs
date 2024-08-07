#![feature(custom_test_frameworks)]
#![test_runner(ferrum_os::utils::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use ferrum_os::memory::BootInfoFrameAllocator;
use ferrum_os::*;
use task::{executor::Executor, keyboard, Task};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    ferrum_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();
    println!("it did not crash");
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    hlt_loop()
}

/// Temporary function used by example_task
async fn async_number() -> u32 {
    42
}
/// Temporary function used to test async/await
/// functionality
async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
