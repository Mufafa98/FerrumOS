use super::madt::MADT;
use super::ACPISDTHeader;
use crate::serial_println;
use alloc::vec::Vec;
pub struct RSDT {
    header: ACPISDTHeader,
    entries: Vec<u32>,
}
use core::ptr;
impl RSDT {
    pub fn new(base_ptr: u32) -> Self {
        let header = ACPISDTHeader::new(base_ptr);
        let mut entries: Vec<u32> = Vec::new();
        let header_size = core::mem::size_of::<ACPISDTHeader>() as u32;
        let num_entries = (header.length - header_size) / 4;
        for i in 0..num_entries {
            let entry =
                unsafe { ptr::read_unaligned((base_ptr + header_size + i * 4) as *const u32) };
            entries.push(entry);
        }
        RSDT { header, entries }
    }
    pub fn is_valid(&self) -> bool {
        let header_sum = self.header.get_sum();
        let mut entries_sum: u32 = 0;
        for entry in self.entries.iter() {
            let entry = *entry;
            let temp_sum = (entry as u8) as u32
                + ((entry >> 8) as u8) as u32
                + ((entry >> 16) as u8) as u32
                + ((entry >> 24) as u8) as u32;
            entries_sum += temp_sum;
        }
        (header_sum + entries_sum) & 0xff == 0
    }
    pub fn list_tables(&self) {
        serial_println!("Listing Tables");
        let total_tables = (self.header.length - core::mem::size_of::<ACPISDTHeader>() as u32) / 4;
        for (i, entry) in self.entries.iter().enumerate() {
            let header = ACPISDTHeader::new(*entry);
            let signature = unsafe { core::str::from_utf8_unchecked(&header.signature) };
            serial_println!("#{}/{} {}", i + 1, total_tables, signature);
        }
    }
    pub fn get_madt(&self) -> Option<MADT> {
        for entry in self.entries.iter() {
            let header = ACPISDTHeader::new(*entry);
            let signature = unsafe { core::str::from_utf8_unchecked(&header.signature) };
            if signature == "APIC" {
                return Some(MADT::new(*entry));
            }
        }
        None
    }
}
