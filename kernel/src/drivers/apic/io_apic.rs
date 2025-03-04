// use crate::serial_println;
#[allow(dead_code)]
pub struct IOAPICStruct {
    io_apic_id: u8,
    io_apic_address: u32,
    global_system_interrupt_base: u32,
}
#[allow(dead_code)]
enum IOAPICReg {
    IOAPICID = 0x00,
    IOAPICVER = 0x01,
    IOAPICARB = 0x02,
    IOAPICREDTBL = 0x10,
}
#[allow(dead_code)]
impl IOAPICStruct {
    pub fn new(io_apic_id: u8, io_apic_address: u32, global_system_interrupt_base: u32) -> Self {
        IOAPICStruct {
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
    /// Set the interrupt vector for the given interrupt
    fn set_red_tbl_vec(&self, interrupt: u8, vector: u8) {
        let entry = IOAPICReg::IOAPICREDTBL as u32 + (interrupt as u32) * 2;
        self.write_register(entry, vector as u32);
    }
}
use lazy_static::lazy_static;
lazy_static! {
    static ref IO_APIC: IOAPICStruct = {
        let ioapic = {
            use crate::drivers::acpi::{rsdp::Rsdp, rsdt::RSDT};
            let rsdp = Rsdp::new();
            let rsdt_header = RSDT::new(rsdp.rsdt_address());
            let madt = rsdt_header.get_madt().expect("No MADT found");
            madt.get_ioapic().expect("No IOAPIC found")
        };

        ioapic
    };
}
pub fn init() {
    use crate::interrupts::InterruptIndexAPIC;
    IO_APIC.set_red_tbl_vec(2, InterruptIndexAPIC::Timer as u8);
    IO_APIC.set_red_tbl_vec(1, InterruptIndexAPIC::Keyboard as u8);
    IO_APIC.set_red_tbl_vec(0x12, InterruptIndexAPIC::HPET as u8);
}
