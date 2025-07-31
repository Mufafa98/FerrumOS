extern crate alloc;
use crate::utils::port::*;
use crate::{failed, ok, println, serial_println, warn};
use alloc::format;
use alloc::vec::Vec;
use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;

pub type BlockIndex = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drive {
    Master = 0,
    Slave = 1,
}
impl Drive {
    fn to_bool(&self) -> bool {
        match self {
            Drive::Master => false,
            Drive::Slave => true,
        }
    }
}

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
#[allow(dead_code)]
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
        let _lower = buf_88.get_bits(0..8);
        let mut mode = 0;
        while upper >> mode & 0b1 == 0 {
            mode += 1;
        }
        UDMA { mode }
    }
}
#[allow(dead_code)]
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

#[derive(Debug)]
pub enum BusError {
    NoDrive,
    DriveNotAta,
    UnableToRead,
    NotInitMembers,
    InvalidBufferSize,
    InvalidBusIdx,
}
#[allow(dead_code)]
pub struct Bus {
    id: u8,
    irq: u8,

    ctrl_base: u16,
    io_base: u16,

    ata_reg: ATARegisters,

    slave: Option<bool>,
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

            ata_reg: ATARegisters::new(io_base, ctrl_base),

            slave: None,
            lba48_supported: None,
            udma: None,
            lba28_adr_sectors: None,
            lba48_adr_sectors: None,
        }
    }

    ///home/Mufafa98_Arch_Laptop/Downloads/ATA8-Command-Set.pdf pp 102
    fn identify(&mut self, drive: Drive) -> Result<(), BusError> {
        use crate::timer::hpet::HPETTimer;
        use crate::timer::Time;
        let hpet_timer = HPETTimer::new();
        let debug_str = format!(
            "ATA {} ",
            if drive == Drive::Master {
                "Master"
            } else {
                "Slave"
            }
        );
        unsafe {
            self.ata_reg.control_register.write(0);
            hpet_timer.sleep(Time::Nanoseconds(400));
            self.ata_reg.sector_count_register.write(0);
            hpet_timer.sleep(Time::Nanoseconds(400));

            self.slave = Some(drive.to_bool());
            if drive == Drive::Master {
                self.ata_reg
                    .drive_register
                    .write(Command::MasterDriveSelector as u8);
            } else {
                self.ata_reg
                    .drive_register
                    .write(Command::SlaveDriveSelector as u8);
            }
            hpet_timer.sleep(Time::Nanoseconds(400));
            self.ata_reg.sector_count_register.write(0);
            self.ata_reg.lba0_register.write(0);
            self.ata_reg.lba1_register.write(0);
            self.ata_reg.lba2_register.write(0);
            self.ata_reg.command_register.write(Command::Identify as u8);
            hpet_timer.sleep(Time::Nanoseconds(400));

            let mut status = self.ata_reg.alternate_status_register.read();
            let error = self.ata_reg.error_register.read();

            if status == 0x00 {
                warn!("{}Drive not found", debug_str);
                return Err(BusError::NoDrive);
            }
            loop {
                status = self.ata_reg.status_register.read();
                if !status.get_bit(Status::BSY as usize) {
                    break;
                }
            }
            let lba_mid = self.ata_reg.lba1_register.read();
            let lba_high = self.ata_reg.lba2_register.read();
            if lba_mid != 0 || lba_high != 0 {
                warn!("{}Drive is not ATA", debug_str,);
                return Err(BusError::DriveNotAta);
            }

            loop {
                status = self.ata_reg.status_register.read();
                if status.get_bit(Status::DRQ as usize) {
                    break;
                }
                if status.get_bit(Status::ERR as usize) {
                    // println!("{}Unable to read from drive", debug_str);
                    failed!("{}Unable to read from drive", debug_str);
                    return Err(BusError::UnableToRead);
                }
            }
            ok!("{}Drive is ready", debug_str);

            let mut buf = [0u16; 256];
            for i in 0..256 {
                buf[i] = self.ata_reg.data_register.read();
            }

            self.udma = Some(UDMA::from_ibf(buf[88]));
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
            ok!("{}Successfully identified drive", debug_str);
            return Ok(());
        }
    }

    fn setup28(&mut self, block: u32) -> Result<(), BusError> {
        if let Some(slave) = self.slave {
            let drive_id = 0xE0 | ((slave as u8) << 4);
            unsafe {
                self.ata_reg
                    .drive_register
                    .write(drive_id | ((block.get_bits(24..28) as u8) & 0x0F));
                self.ata_reg.sector_count_register.write(1);
                self.ata_reg.lba0_register.write(block.get_bits(0..8) as u8);
                self.ata_reg
                    .lba1_register
                    .write(block.get_bits(8..16) as u8);
                self.ata_reg
                    .lba2_register
                    .write(block.get_bits(16..24) as u8);
            }
            return Ok(());
        } else {
            // panic!("Slave drive not set. Maybe init() was not called?");
            return Err(BusError::NotInitMembers);
        }
    }

    fn is_busy(&self) -> bool {
        unsafe {
            self.ata_reg
                .status_register
                .read()
                .get_bit(Status::BSY as usize)
        }
    }

    fn write_command(&mut self, command: Command) {
        unsafe {
            self.ata_reg.command_register.write(command as u8);
        }
    }

    fn read_data(&mut self) -> u16 {
        unsafe { self.ata_reg.data_register.read() }
    }

    fn write_data(&mut self, data: u16) {
        unsafe { self.ata_reg.data_register.write(data) }
    }

    pub fn read(&mut self, block: BlockIndex, buf: &mut [u8]) -> Result<(), BusError> {
        if buf.len() != 512 {
            // panic!("Buffer size must be 512 bytes");
            return Err(BusError::InvalidBufferSize);
        }
        let setup_result = self.setup28(block as u32);
        if let Ok(_) = setup_result {
            self.write_command(Command::Read);
            while self.is_busy() {}

            for i in 0..256 {
                let data = self.read_data();
                buf[i * 2] = (data & 0xFF) as u8;
                buf[i * 2 + 1] = (data >> 8) as u8;
            }
        }
        return setup_result;
    }

    pub fn write(&mut self, block: BlockIndex, buf: &[u8]) -> Result<(), BusError> {
        if buf.len() != 512 {
            // panic!("Buffer size must be 512 bytes");
            return Err(BusError::InvalidBufferSize);
        }
        let setup_result = self.setup28(block as u32);
        if let Ok(_) = setup_result {
            self.write_command(Command::Write);
            while self.is_busy() {}
            for i in 0..256 {
                let mut data = 0 as u16;
                data.set_bits(0..8, buf[i * 2] as u16);
                data.set_bits(8..16, buf[i * 2 + 1] as u16);

                self.write_data(data);
                self.write_command(Command::CacheFlush);
                while self.is_busy() {}
            }
        }
        return setup_result;
    }

    pub fn get_block(&self, address: u32) -> BlockIndex {
        if let Some(lba28_adr_sectors) = self.lba28_adr_sectors {
            return (address / lba28_adr_sectors) as BlockIndex;
        }
        0
    }
}

lazy_static! {
    static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

pub fn read(bus: u8, block: BlockIndex, buf: &mut [u8]) -> Result<(), BusError> {
    if bus as usize >= BUSES.lock().len() {
        return Err(BusError::InvalidBusIdx);
    }
    let mut buses = BUSES.lock();
    buses[bus as usize].read(block, buf)
}

pub fn write(bus: u8, block: BlockIndex, buf: &[u8]) -> Result<(), BusError> {
    if bus as usize >= BUSES.lock().len() {
        return Err(BusError::InvalidBusIdx);
    }
    let mut buses = BUSES.lock();
    buses[bus as usize].write(block, buf)
}

pub fn init() {
    let mut master = Bus::new(0, 0x1F0, 0x3F6, 14);
    if let Err(err) = master.identify(Drive::Master) {
        serial_println!("Master Error: {:?}", err);
    } else {
        BUSES.lock().push(master);
    }
    let mut slave = Bus::new(1, 0x170, 0x376, 15);
    if let Err(err) = slave.identify(Drive::Slave) {
        serial_println!("Slave Error: {:?}", err);
    } else {
        BUSES.lock().push(slave);
    }
}
