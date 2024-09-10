use crate::println;
use crate::utils::msr::*;
use crate::utils::registers::*;
static IA32_APIC_BASE_MSR: u32 = 0x1b;

enum LAPICReg {
    ID = 0x20,
    Version = 0x30,
    EOI = 0xb0,
    SpuriousInterruptVector = 0xf0,
}

#[derive(Debug)]
pub struct LocalAPIC {
    bsc: bool,         // Boot strap processor
    is_enabled: bool,  // APIC Enabled
    base_address: u64, // APIC Base Address
}

impl LocalAPIC {
    pub fn init() -> Self {
        let register = read_msr(IA32_APIC_BASE_MSR);
        let bsc = (register >> 8) & 1 == 1;
        let is_enabled = (register >> 11) & 1 == 1;
        let base = register >> 12;
        // REMOVE
        //serial_println!("addr: {:x}", base << 12);
        LocalAPIC {
            bsc,
            is_enabled,
            base_address: base << 12,
        }
    }
    fn read_register(&self, register: LAPICReg) -> u32 {
        let offset = register as u64;
        unsafe {
            return *((self.base_address + offset) as *const u32);
        }
    }
    fn write_register(&self, register: LAPICReg, value: u32) {
        let offset = register as u64;
        unsafe {
            let ptr = (self.base_address + offset) as *mut u32;
            *ptr = value;
        }
    }
    fn get_version(&self) -> u32 {
        self.read_register(LAPICReg::Version)
    }
    fn get_id(&self) -> u32 {
        self.read_register(LAPICReg::ID)
    }
    fn set_eoi(&self) {
        self.write_register(LAPICReg::EOI, 0);
    }
    // TO DO Remove temp function
    fn temp_spurious_interrupt_vector(&self) {
        let offset = 0xf0;
        unsafe {
            let ptr = (self.base_address + offset) as *mut u32;
            *ptr = (*ptr) | 0x1FF;
            // REMOVE
            //serial_println!("SIV: {:b}", *ptr);
        }
    }
    fn disable_pic() {
        // const PIC_COMMAND_MASTER: u16 = 0x20;
        // const PIC_COMMAND_SLAVE: u16 = 0xa0;
        const PIC_DATA_MASTER: u16 = 0x21;
        const PIC_DATA_SLAVE: u16 = 0xa1;
        outb(PIC_DATA_MASTER, 0xff);
        outb(PIC_DATA_SLAVE, 0xff);
    }
    pub fn enable() {
        use crate::utils::cpuid::check_apic;
        if check_apic() {
            LocalAPIC::disable_pic();
            let local_apic = LocalAPIC::init();
            local_apic.temp_spurious_interrupt_vector();
            println!("LAPIC[{}] Enabled", local_apic.get_id());
        } else {
            panic!("APIC Not Supported");
        }
    }
}
