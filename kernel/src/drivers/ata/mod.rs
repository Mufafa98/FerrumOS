extern crate alloc;
// TODO ???????

use crate::io::serial;
use crate::utils::port::*;
use crate::{hlt_loop, print, serial_print, serial_println};
use alloc::vec::Vec;
use bit_field::BitField;
use core::arch::asm;
use core::error;
use core::task::Poll;
use lazy_static::lazy_static;
use spin::Mutex;

use core::hint::spin_loop;

pub type BlockIndex = u32;

#[repr(u16)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    CacheFlush = 0xE7,
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

#[derive(Debug)]
struct UDMA {
    mode: u8,
}
impl UDMA {
    /// Constructor for UDMA that is called after the IDENTIFY command on the
    /// 88th u16 of the buffer
    ///
    /// # Safety
    /// This function is unsafe because the user needs to ensure that the
    /// provded buffer is indeed the 88th u16 of the buffer. Otherwise, it
    /// may cause undefined behavior.
    unsafe fn from_ibf(buf_88: u16) -> Self {
        // buf_88 should look like:
        // 0bRXXXXXXX_RXXXXXXX
        let upper = buf_88.get_bits(8..16);
        let lower = buf_88.get_bits(0..8);
        let mut mode = 0;
        while upper >> mode & 0b1 == 0 {
            mode += 1;
        }
        UDMA { mode }
    }
}

struct ATARegisters {
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
impl ATARegisters {
    fn new(io_base: u16, ctrl_base: u16) -> Self {
        Self {
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
}

pub struct Bus {
    id: u8,
    irq: u8,

    ctrl_base: u16,
    io_base: u16,

    registers: ATARegisters,

    lba48_supported: Option<bool>,
    udma: Option<UDMA>,
    lba28_adr_sectors: Option<u32>,
    lba48_adr_sectors: Option<u64>,
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self {
            id,
            irq,

            ctrl_base,
            io_base,

            registers: ATARegisters::new(io_base, ctrl_base),

            lba48_supported: None,
            udma: None,
            lba28_adr_sectors: None,
            lba48_adr_sectors: None,
        }
    }

    ///home/Mufafa98_Arch_Laptop/Downloads/ATA8-Command-Set.pdf pp 102
    unsafe fn identify(&mut self, master_drive: bool) {
        use crate::timer::hpet::HPETTimer;
        use crate::timer::Time;
        let hpet_timer = HPETTimer::new();

        self.registers.control_register.write(0);
        hpet_timer.sleep(Time::Nanoseconds(400));
        self.registers.sector_count_register.write(0);
        hpet_timer.sleep(Time::Nanoseconds(400));

        self.registers.drive_register.write(if master_drive {
            serial_println!("Master Drive");
            Command::MasterDriveSelector as u8
        } else {
            serial_println!("Slave Drive");
            Command::SlaveDriveSelector as u8
        });
        hpet_timer.sleep(Time::Nanoseconds(400));
        self.registers.sector_count_register.write(0);
        self.registers.lba0_register.write(0);
        self.registers.lba1_register.write(0);
        self.registers.lba2_register.write(0);
        serial_println!("Setting up registers");
        self.registers
            .command_register
            .write(Command::Identify as u8);
        serial_println!("Sent Identify Command");
        hpet_timer.sleep(Time::Nanoseconds(400));

        let mut status = self.registers.alternate_status_register.read();
        let error = self.registers.error_register.read();

        if status == 0x00 {
            panic!("No drive found");
            return;
        }
        loop {
            status = self.registers.status_register.read();
            if !status.get_bit(Status::BSY as usize) {
                break;
            }
        }
        serial_println!("Drive found");
        let lba_mid = self.registers.lba1_register.read();
        let lba_high = self.registers.lba2_register.read();
        if lba_mid != 0 || lba_high != 0 {
            panic!("Drive is not ATA");
        }

        loop {
            status = self.registers.status_register.read();
            if status.get_bit(Status::DRQ as usize) {
                break;
            }
            if status.get_bit(Status::ERR as usize) {
                panic!("Error reading drive");
            }
        }
        serial_println!("Drive is ready");

        let mut buf = [0u16; 256];
        for i in 0..256 {
            buf[i] = self.registers.data_register.read();
            // serial_println!("buf[{}]: 0x{:04X}", i, buf[i]);
        }

        unsafe {
            self.udma = Some(UDMA::from_ibf(buf[88]));
        }
        let mut lba32 = 0;
        lba32 |= (buf[61] as u32) << 16;
        lba32 |= (buf[60] as u32) << 0;
        self.lba28_adr_sectors = Some(lba32);
        self.lba48_supported = Some(buf[83] >> 10 & 0b1 == 1);
        if self.lba48_supported.unwrap() {
            let mut lba48 = 0;
            lba48 |= (buf[103] as u64) << 48;
            lba48 |= (buf[102] as u64) << 32;
            lba48 |= (buf[101] as u64) << 16;
            lba48 |= (buf[100] as u64) << 0;
            self.lba48_adr_sectors = Some(lba48);
        }
        serial_println!("Drive Info:");
        serial_println!("  UDMA Mode: {:?}", self.udma);
        serial_println!("  LBA28: {}", self.lba28_adr_sectors.unwrap());
        if self.lba48_supported.unwrap() {
            serial_println!("  LBA48: {}", self.lba48_adr_sectors.unwrap());
        } else {
            serial_println!("  LBA48: Not supported");
        }
    }

    fn setup28(&mut self, slave: bool, block: u32) {
        let drive_id = 0xE0 | ((slave as u8) << 4);
        unsafe {
            self.registers
                .drive_register
                .write(drive_id | ((block.get_bits(24..28) as u8) & 0x0F));
            self.registers.sector_count_register.write(1);
            self.registers
                .lba0_register
                .write(block.get_bits(0..8) as u8);
            self.registers
                .lba1_register
                .write(block.get_bits(8..16) as u8);
            self.registers
                .lba2_register
                .write(block.get_bits(16..24) as u8);
        }
    }

    pub fn read(&mut self, slave: bool, block: BlockIndex, buf: &mut [u8]) {
        if buf.len() != 512 {
            panic!("Buffer size must be 512 bytes");
        }
        self.setup28(slave, block as u32);
        unsafe {
            self.registers.command_register.write(Command::Read as u8);
        }
        while unsafe {
            self.registers
                .status_register
                .read()
                .get_bit(Status::BSY as usize)
        } {}
        for i in 0..256 {
            let data = unsafe { self.registers.data_register.read() };
            buf[i * 2] = (data & 0xFF) as u8;
            buf[i * 2 + 1] = (data >> 8) as u8;
        }
    }

    pub fn write(&mut self, slave: bool, block: BlockIndex, buf: &[u8]) {
        if buf.len() != 512 {
            panic!("Buffer size must be 512 bytes");
        }
        self.setup28(slave, block as u32);
        unsafe {
            self.registers.command_register.write(Command::Write as u8);
        }
        while unsafe {
            self.registers
                .status_register
                .read()
                .get_bit(Status::BSY as usize)
        } {}
        for i in 0..256 {
            let mut data = 0 as u16;
            data.set_bits(0..8, buf[i * 2] as u16);
            data.set_bits(8..16, buf[i * 2 + 1] as u16);
            serial_println!("Writing data: 0x{:04X}", data);
            unsafe { self.registers.data_register.write(data) };
            unsafe {
                self.registers
                    .command_register
                    .write(Command::CacheFlush as u8);
            }
        }
    }
}

lazy_static! {
    static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

pub fn read(bus: u8, drive: u8, block: BlockIndex, buf: &mut [u8]) {
    let mut buses = BUSES.lock();
    //log!("Reading Block 0x{:08X}\n", block);
    // buses[bus as usize].read(drive, block, buf);
}

pub fn init() {
    let mut bus = Bus::new(0, 0x1F0, 0x3F6, 14);
    unsafe { bus.identify(true) };
    let mut buf = [0u8; 512];
    bus.read(false, 0, &mut buf);
    for i in 0..512 {
        serial_print!("{:02X} ", buf[i]);
        if i % 16 == 15 {
            serial_print!("\n");
        }
    }
    bus.read(false, 1, &mut buf);
    for i in 0..512 {
        serial_print!("{:02X} ", buf[i]);
        if i % 16 == 15 {
            serial_print!("\n");
        }
    }
    // buf = [0x45; 512];
    // bus.write(false, 0, &buf);
    // bus.write(false, 1, &buf);

    // let mut buses = BUSES.lock();
    // let mut master = Bus::new(0, 0x1F0, 0x3F6, 14);

    // // master.reset();
    // serial_println!("ATA Reset Done");
    // master.print_type();
    // buses.push(master);

    // master.write_command(Command::Identify);
    // buses.push(Bus::new(1, 0x170, 0x376, 15));
    // serial_println!("ATA initialized");
}
