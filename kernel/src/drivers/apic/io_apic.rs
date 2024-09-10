// use crate::serial_println;

pub struct IOAPIC {
    io_apic_id: u8,
    io_apic_address: u32,
    global_system_interrupt_base: u32,
}
enum IOAPICReg {
    IOAPICID = 0x00,
    IOAPICVER = 0x01,
    IOAPICARB = 0x02,
    IOAPICREDTBL = 0x10,
}
#[allow(dead_code)]
impl IOAPIC {
    pub fn new(io_apic_id: u8, io_apic_address: u32, global_system_interrupt_base: u32) -> Self {
        IOAPIC {
            io_apic_id,
            io_apic_address,
            global_system_interrupt_base,
        }
    }
    fn read_register(&self, register: u32) -> u32 {
        unsafe {
            let io_reg_sel = self.io_apic_address as *mut u32;
            let io_reg_win = (self.io_apic_address + 0x10) as *mut u32;
            *io_reg_sel = register;
            *io_reg_win
        }
    }
    fn write_register(&self, register: u32, value: u32) {
        unsafe {
            let io_reg_sel = self.io_apic_address as *mut u32;
            let io_reg_win = (self.io_apic_address + 0x10) as *mut u32;
            *io_reg_sel = register;
            *io_reg_win = value;
        }
    }
    fn set_mask(&self, interrupt: u8, mask: bool) {
        let reg = (IOAPICReg::IOAPICREDTBL as u32 + (interrupt as u32) * 2) as u32;
        let value = self.read_register(reg);

        if value & 0x10000 != 0 {
            if !mask {
                self.write_register(reg, value & !0x10000);
            }
        } else {
            if mask {
                self.write_register(reg, value | 0x10000);
            }
        }
    }
    pub fn enable_ioapic() {
        use crate::drivers::acpi::{rsdp::Rsdp, rsdt::RSDT};
        let rsdp = Rsdp::new();
        // serial_println!("RSDP Valid: {}", rsdp.is_valid());
        let rsdt_header = RSDT::new(rsdp.rsdt_address());
        // REMOVE
        // serial_println!("RSDT Valid: {}", rsdt_header.is_valid());
        // rsdt_header.list_tables();
        let madt = rsdt_header.get_madt().expect("No MADT found");
        let ioapic = madt.get_ioapic().expect("No IOAPIC found");
        // TO DO FIX with general purpose
        ioapic.write_register(IOAPICReg::IOAPICREDTBL as u32 + 4, 32);
        // let entries = madt.entries;
        // for entry in entries.iter() {
        //     serial_println!("{:?}", entry);
        // }
    }
}
