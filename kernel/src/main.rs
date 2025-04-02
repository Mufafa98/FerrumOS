#![no_std]
#![no_main]

use alloc::string::{String, ToString};
use ferrum_os::*;
use io::serial;
use task::{executor, keyboard, Task};
use timer::Time;
use utils::registers::{inb, outb};

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
    // use drivers::ata::test;
    ferrum_os::init();
    welcome();

    // serial_println!("Testing");
    // hpet();
    // serial_println!("Done");

    ata();
    // test();
    // pci();
    let mut executor = executor::Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

fn calibrate() {
    use drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
    use interrupts::InterruptIndexAPIC;
    let ticks_per_ms = lapic_tick_per_ms();
    LOCAL_APIC.write_register(LAPICReg::TimerLVT, InterruptIndexAPIC::LAPICTimer.as_u32());
    LOCAL_APIC.write_register(LAPICReg::TimerDCnf, 0x3);
    LOCAL_APIC.write_register(LAPICReg::TimerICnt, ticks_per_ms);
    // serial_println!("start");
    // temp_sleep(1000);
    // serial_println!("end");
    serial_println!("Ticks per ms: {}", ticks_per_ms);
}
fn lapic_tick_per_ms() -> u32 {
    use drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
    use timer::pit::PIT;
    let measure_duration: u32 = 1000;
    LOCAL_APIC.write_register(LAPICReg::TimerDCnf, 0x3);
    LOCAL_APIC.write_register(LAPICReg::TimerICnt, 0xFFFFFFFF);
    PIT::sleep(measure_duration as u64);
    let ticks_raw = 0xFFFFFFFF - LOCAL_APIC.read_register(LAPICReg::TimerCCnt);
    ticks_raw / measure_duration
}
fn welcome() {
    let title = "FerrumOs has started";
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
}
fn _test_timer_old() {
    use timer::lapic::*;
    use timer::pit::PIT;
    lapic_calibrate();
    serial_println!("start");
    let start = PIT::get_counter();
    LAPICTimer::sleep(100);
    // timer::pit::PIT::sleep(1000);
    let end = PIT::get_counter();
    serial_println!("end");
    serial_println!("Ticks: {}", end - start);
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

use core::{arch::asm, char, pin::Pin, time};

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
#[allow(unconditional_recursion)]
fn _inf_rec() {
    _inf_rec();
    x86_64::instructions::hlt();
}

fn hpet() {
    // use drivers::acpi::{
    //     hpet::{HPETRegisters, HPET},
    //     rsdp::Rsdp,
    //     rsdt::RSDT,
    // };
    // let rsdp = Rsdp::new();
    // let rsdt_table = RSDT::new(rsdp.rsdt_address());
    // let hpet = rsdt_table.get_hpet().unwrap();
    // let test_time = 100_000_000;
    // hpet.set_timer_n_comparator(2, test_time);
    // hpet.get_timer_n_config(2).set_interrupt_idx(0x12);
    // hpet.get_timer_n_config(2).enable_interrupt();
    // hpet.enable();
    use crate::timer::hpet::HPETTimer;
    let timer = HPETTimer::new();
    timer.sleep(Time::Nanoseconds(10000));
}

fn ata() {
    const ATA_BLOCK_SIZE: usize = 512;
    drivers::ata::init();
    let mut buffer: [u8; ATA_BLOCK_SIZE] = [0; ATA_BLOCK_SIZE];
    // drivers::ata::read(0, 0, 0, &mut buffer);
    // for i in 0..ATA_BLOCK_SIZE {
    //     serial_print!("{:X} ", buffer[i]);
    // }
    serial_println!("\nDone");
}
