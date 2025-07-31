use super::ACPISDTHeader;
use crate::drivers::apic::io_apic::IOAPICStruct;
use alloc::vec::Vec;
#[allow(dead_code)]
#[derive(Debug)]
enum MADTEntry {
    LocalApicEntry {
        // Type 0
        processor_id: u8, // ACPI Processor ID
        apic_id: u8,      // Local APIC ID
        flags: u32,       // Flags
                          //    bit = 0: Processor Enabled
                          //    bit = 1: Online Capable
    },
    IoApicEntry {
        // Type 1
        ioapic_id: u8,                     // IO APIC ID
        ioapic_address: u32,               // IO APIC Address
        global_system_interrupt_base: u32, // Global System Interrupt Base
    },
    IoApicInterruptSourceOverrideEntry {
        // Type 2
        bus_source: u8,
        irq_source: u8,
        global_system_interrupt: u32,
        flags: u16,
    },
    // IoApicNmiSourceEntry {
    //     // Type 3
    //     nmi_source: u8,
    //     flags: u16,
    //     global_system_interrupt: u32,
    // },
    LocalApicNmiEntry {
        // Type 4
        processor_id: u8,
        flags: u16,
        local_apic_lint: u8,
    },
    // LocalApicAddressOverrideEntry {
    //     local_apic_address: u64,
    // },
    // PeocessorLocalX2ApicEntry {
    //     x2apic_id: u32,
    //     flags: u32,
    //     acpi_processor_uid: u32,
    // },
}
#[allow(dead_code)]
pub struct MADT {
    header: ACPISDTHeader,
    local_apic_address: u32,
    flags: u32,
    entries_offset: u32,
    entries: Vec<MADTEntry>,
}
impl MADT {
    pub fn new(base_ptr: u32) -> Self {
        use core::ptr;

        let header = ACPISDTHeader::new(base_ptr);
        let header_size = core::mem::size_of::<ACPISDTHeader>() as u32;
        let local_apic_address =
            unsafe { ptr::read_unaligned((base_ptr + header_size) as *const u32) };
        let flags = unsafe { ptr::read_unaligned((base_ptr + header_size + 4) as *const u32) };
        let mut entries_offset = (base_ptr + header_size + 8) as u32;
        let mut entries: Vec<MADTEntry> = Vec::new();
        let mut left_to_read = header.length - header_size;
        while left_to_read > 0 {
            let entry_type = unsafe { ptr::read_unaligned(entries_offset as *const u8) };
            let record_length = unsafe { ptr::read_unaligned((entries_offset + 1) as *const u8) };

            match entry_type {
                0 => {
                    let processor_id =
                        unsafe { ptr::read_unaligned((entries_offset + 2) as *const u8) };
                    let apic_id = unsafe { ptr::read_unaligned((entries_offset + 3) as *const u8) };
                    let flags = unsafe { ptr::read_unaligned((entries_offset + 4) as *const u32) };
                    entries_offset += record_length as u32;

                    let entry = MADTEntry::LocalApicEntry {
                        processor_id,
                        apic_id,
                        flags,
                    };
                    entries.push(entry);
                }
                1 => {
                    let ioapic_id =
                        unsafe { ptr::read_unaligned((entries_offset + 2) as *const u8) };
                    let ioapic_address =
                        unsafe { ptr::read_unaligned((entries_offset + 4) as *const u32) };
                    let global_system_interrupt_base =
                        unsafe { ptr::read_unaligned((entries_offset + 8) as *const u32) };
                    entries_offset += record_length as u32;
                    let entry = MADTEntry::IoApicEntry {
                        ioapic_id,
                        ioapic_address,
                        global_system_interrupt_base,
                    };
                    entries.push(entry);
                }
                2 => {
                    let bus_source =
                        unsafe { ptr::read_unaligned((entries_offset + 2) as *const u8) };
                    let irq_source =
                        unsafe { ptr::read_unaligned((entries_offset + 3) as *const u8) };
                    let global_system_interrupt =
                        unsafe { ptr::read_unaligned((entries_offset + 4) as *const u32) };
                    let flags = unsafe { ptr::read_unaligned((entries_offset + 8) as *const u16) };
                    entries_offset += record_length as u32;
                    let entry = MADTEntry::IoApicInterruptSourceOverrideEntry {
                        bus_source,
                        irq_source,
                        global_system_interrupt,
                        flags,
                    };
                    entries.push(entry);
                }
                4 => {
                    let processor_id =
                        unsafe { ptr::read_unaligned((entries_offset + 2) as *const u8) };
                    let flags = unsafe { ptr::read_unaligned((entries_offset + 3) as *const u16) };
                    let local_apic_lint =
                        unsafe { ptr::read_unaligned((entries_offset + 5) as *const u8) };
                    entries_offset += record_length as u32;

                    let entry = MADTEntry::LocalApicNmiEntry {
                        processor_id,
                        flags,
                        local_apic_lint,
                    };
                    entries.push(entry);
                }
                _ => {
                    entries_offset += record_length as u32;
                    // println!("Unknown Entry Type In MADT Table: {}", entry_type);
                }
            }
            //serial_println!("Entry: {:?}", entries.last().unwrap());
            if record_length as u32 > left_to_read {
                left_to_read = 0;
            } else {
                left_to_read -= record_length as u32;
            }
        }
        // for entry in entries.iter() {
        //     serial_println!("{:?}", entry);
        // }
        entries.shrink_to_fit();
        MADT {
            header,
            local_apic_address,
            flags,
            entries_offset,
            entries,
        }
    }
    pub fn get_ioapic(&self) -> Option<IOAPICStruct> {
        for entry in self.entries.iter() {
            match entry {
                MADTEntry::IoApicEntry {
                    ioapic_id,
                    ioapic_address,
                    global_system_interrupt_base,
                } => {
                    return Some(IOAPICStruct::new(
                        *ioapic_id,
                        *ioapic_address,
                        *global_system_interrupt_base,
                    ));
                }
                _ => {}
            }
        }
        None
    }
}
