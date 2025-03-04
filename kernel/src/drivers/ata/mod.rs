extern crate alloc;
// TODO ???????

use crate::utils::port::*;
use crate::{hlt_loop, serial_print, serial_println};
use alloc::vec::Vec;
use bit_field::BitField;
use core::arch::asm;
use core::task::Poll;
use lazy_static::lazy_static;
use spin::Mutex;

use core::hint::spin_loop;

pub type BlockIndex = u32;

#[repr(u16)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Identify = 0xEC,
    MasterDriveSelector = 0xA0,
    SlaveDriveSelector = 0xB0,
}

#[allow(dead_code)]
#[repr(usize)]
enum Status {
    ERR = 0,
    IDX = 1,
    CORR = 2,
    DRQ = 3,
    SRV = 4,
    DF = 5,
    RDY = 6,
    BSY = 7,
}
pub struct Bus {
    id: u8,
    irq: u8,

    ctrl_base: u16,
    io_base: u16,

    data_register: Port<u16>,
    error_register: PortReadOnly<u8>,
    features_register: PortWriteOnly<u8>,
    sector_count_register: Port<u8>,
    lba0_register: Port<u8>,
    lba1_register: Port<u8>,
    lba2_register: Port<u8>,
    drive_register: Port<u8>,
    status_register: PortReadOnly<u8>,
    command_register: PortWriteOnly<u8>,

    alternate_status_register: PortReadOnly<u8>,
    control_register: PortWriteOnly<u8>,
    drive_blockess_register: PortReadOnly<u8>,
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self {
            id,
            irq,

            ctrl_base,
            io_base,

            data_register: Port::new(io_base + 0),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),

            alternate_status_register: PortReadOnly::new(ctrl_base + 0),
            control_register: PortWriteOnly::new(ctrl_base + 0),
            drive_blockess_register: PortReadOnly::new(ctrl_base + 1),
        }
    }
    fn write_command(&mut self, command: Command) {
        unsafe {
            self.command_register.write(command as u8);
        }
    }
    fn reset(&mut self) {
        unsafe {
            use crate::timer::{hpet::HPETTimer, Time};
            let timer = HPETTimer::new();
            use crate::interrupts::handlers::*;
            self.control_register.write(4); // Set SRST bit
            timer.sleep(Time::Nanoseconds(400));
            self.control_register.write(0); // Then clear it
            timer.sleep(Time::Nanoseconds(400));
        }
    }
    pub fn print_type(&mut self) {
        unsafe {
            self.drive_register.write(0xA0);
            self.sector_count_register.write(0);
            self.lba0_register.write(0);
            self.lba1_register.write(0);
            self.lba2_register.write(0);
            self.command_register.write(0xEC);
            serial_println!("Sent Identify Command");
            if self.status_register.read() != 0 {
                while self.status_register.read().get_bit(Status::BSY as usize) {
                    serial_println!("Status busy");
                    // hlt_loop();
                }
                serial_println!("Not busy anymore");
            }
            // if self.status_register.read() == 0 {
            //     panic!("Drive Does Not Exist")
            // }
            // while self.status_register.read().get_bit(Status::BSY as usize) {
            //     serial_println!("{:b}", self.status_register.read());
            //     // if self.lba1_register.read() != 0 || self.lba2_register.read() != 0 {
            //     //     serial_println!("Drive is not ATA");
            //     //     break;
            //     // }
            //     serial_println!("Status busy");
            //     hlt_loop();
            // }
            // serial_println!("Not busy anymore");
            // serial_println!("Errors: {}", self.error_register.read());
        }
    }
}
fn sleep_ticks(ticks: usize) {
    for _ in 0..=ticks {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
lazy_static! {
    static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

pub fn test() {
    serial_println!("Testing ATA");
    init();
}

pub fn read(bus: u8, drive: u8, block: BlockIndex, buf: &mut [u8]) {
    let mut buses = BUSES.lock();
    //log!("Reading Block 0x{:08X}\n", block);
    // buses[bus as usize].read(drive, block, buf);
}

pub fn init() {
    let mut buses = BUSES.lock();
    let mut master = Bus::new(0, 0x1F0, 0x3F6, 14);
    master.reset();
    master.print_type();
    buses.push(master);
    // master.write_command(Command::Identify);
    // buses.push(Bus::new(1, 0x170, 0x376, 15));
    // serial_println!("ATA initialized");
}
