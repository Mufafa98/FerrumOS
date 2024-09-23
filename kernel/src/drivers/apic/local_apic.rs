use crate::println;
use crate::serial_println;
use crate::utils::msr::*;
use crate::utils::registers::*;
static IA32_APIC_BASE_MSR: u32 = 0x1b;
#[allow(dead_code)]
pub enum LAPICReg {
    ID = 0x20,
    Version = 0x30,
    EOI = 0xb0,
    TimerLVT = 0x320,
    TimerICnt = 0x380, // initial count
    TimerCCnt = 0x390, // current count
    TimerDCnf = 0x3e0, // divide configuration
    SpuriousInterruptVector = 0xf0,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LocalAPIC {
    bsc: bool,         // Boot strap processor
    is_enabled: bool,  // APIC Enabled
    base_address: u64, // APIC Base Address
}
#[allow(dead_code)]
impl LocalAPIC {
    pub fn init() -> Self {
        let register = read_msr(IA32_APIC_BASE_MSR);
        let bsc = (register >> 8) & 1 == 1;
        let is_enabled = (register >> 11) & 1 == 1;
        let base = register >> 12;
        LocalAPIC {
            bsc,
            is_enabled,
            base_address: base << 12,
        }
    }
    pub fn read_register(&self, register: LAPICReg) -> u32 {
        let offset = register as u64;
        unsafe {
            return *((self.base_address + offset) as *const u32);
        }
    }
    pub fn write_register(&self, register: LAPICReg, value: u32) {
        let offset = register as u64;
        unsafe {
            let ptr = (self.base_address + offset) as *mut u32;
            *ptr = value;
        }
    }
    // TO DO consider removing those in favor of read_register and write_register
    fn get_version(&self) -> u32 {
        self.read_register(LAPICReg::Version)
    }
    fn get_id(&self) -> u32 {
        self.read_register(LAPICReg::ID)
    }
    pub fn set_eoi(&self) {
        self.write_register(LAPICReg::EOI, 0);
    }
    fn set_spurious_interrupt_vector(&self, value: u32) {
        self.write_register(LAPICReg::SpuriousInterruptVector, value)
    }
    fn get_spurious_interrupt_vector(&self) -> u32 {
        self.read_register(LAPICReg::SpuriousInterruptVector)
    }
    fn disable_pic() {
        // const PIC_COMMAND_MASTER: u16 = 0x20;
        // const PIC_COMMAND_SLAVE: u16 = 0xa0;
        const PIC_DATA_MASTER: u16 = 0x21;
        const PIC_DATA_SLAVE: u16 = 0xa1;
        outb(PIC_DATA_MASTER, 0xff);
        outb(PIC_DATA_SLAVE, 0xff);
    }
    pub fn enable() {}
}
use lazy_static::lazy_static;
lazy_static! {
    pub static ref LOCAL_APIC: LocalAPIC = {
        use crate::utils::cpuid::check_apic;
        if !check_apic() {
            panic!("APIC Not Supported");
        }
        LocalAPIC::disable_pic();
        let local_apic = LocalAPIC::init();
        local_apic
    };
}

pub fn init() {
    // 0x100 -> Enable LAPIC
    // 0xFF  -> Set the vector
    let spourious_interrupt_vector = LOCAL_APIC.get_spurious_interrupt_vector() | 0x1FF;
    LOCAL_APIC.set_spurious_interrupt_vector(spourious_interrupt_vector);
    println!("LAPIC[{}] Enabled", LOCAL_APIC.get_id());
}
