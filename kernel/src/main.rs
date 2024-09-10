#![no_std]
#![no_main]

use alloc::{
    fmt::format,
    format,
    string::{String, ToString},
    vec::Vec,
};
use ferrum_os::*;

use io::serial;
use task::{executor, keyboard, Task};
use x86_64::{
    addr,
    instructions::interrupts::enable,
    structures::paging::{Mapper, Page, PhysFrame, Size4KiB, Translate},
    PhysAddr, VirtAddr,
};

extern crate alloc;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    ferrum_os::init();
    let title = "FerrumOs has started";
    let separator = "-".repeat(title.len());
    let mut features = "".to_string();
    #[cfg(feature = "test")]
    features.push_str("\n Test");
    #[cfg(not(feature = "test"))]
    features.push_str("\n Default");
    serial_println!(
        "<{separator}>\n {} \n [Features]:{} \n<{separator}>",
        title,
        features,
        separator = "-".repeat(title.len())
    );
    println!("Welcome to FerrumOs");
    unsafe {
        // let mut rbx: i64 = 0;
        // let mut rdx: i64 = 0;
        // let mut rcx: i64 = 0;
        // asm!("mov rax, 0x0", "cpuid",);
        // asm!("mov {rbx}, rbx",rbx = out(reg) rbx,);
        // asm!("mov {rdx}, rdx",rdx = out(reg) rdx,);
        // asm!("mov {rcx}, rcx",rcx = out(reg) rcx,);
        // let mut string: String = String::new();
        // string.push_str("CPUID: ");
        // string.push_str(i64_to_str(rbx).as_str());
        // string.push_str(i64_to_str(rdx).as_str());
        // string.push_str(i64_to_str(rcx).as_str());

        // println!("{}", string);

        // if check_apic() {
        //     println!("APIC Supported");
        //     // enable_apic();
        // } else {
        //     panic!("APIC Not Supported");
        // }
        enable_apic();
        // loop {
        //     serial_println!("Start {:?}", crate::interrupts::handlers::COUNTER);
        // }
    }
    let mut i = 'a' as u8;
    loop {
        print!("{}", i as char);
        i += 1;
        if i == 'z' as u8 {
            i = i % ('z' as u8) + 'a' as u8;
        }
    }
    // Async/await
    let mut executor = executor::Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

fn enable_apic() {
    use crate::drivers::apic::{io_apic::IOAPIC, local_apic::LocalAPIC};
    LocalAPIC::enable();
    IOAPIC::enable_ioapic();
    timer();
}

fn timer() {
    use timer::pit::PIT;
    let mut timer = PIT::new();
    timer.set_timer(1);
    timer.start();
}

fn _i64_to_str(i: i64) -> String {
    let mut string = String::new();
    string.push((i & 0xff) as u8 as char);
    string.push(((i >> 8) & 0xff) as u8 as char);
    string.push(((i >> 16) & 0xff) as u8 as char);
    string.push(((i >> 24) & 0xff) as u8 as char);
    string
}

async fn _async_hello() {
    for _ in 0..5 {
        println!("Hello from async_hello");
    }
}
async fn _async_world() {
    for _ in 0..5 {
        println!("World from async_world");
    }
}

use core::{arch::asm, char, ptr, sync::atomic::AtomicU64};

#[allow(dead_code)]
fn _heap_test_debug() {
    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
    let heap_value = Box::new(41);
    serial_println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..5000 {
        vec.push(i);
    }
    serial_println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    serial_println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    serial_println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );
}
// TO DO : Throw error when stack overflow
fn _inf_rec() {
    _inf_rec();
    x86_64::instructions::hlt();
}
