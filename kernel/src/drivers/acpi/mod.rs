pub mod hpet;
mod madt;
pub mod rsdp;
pub mod rsdt;

#[repr(C)]
#[derive(Debug)]
struct ACPISDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}
use core::ptr;
impl ACPISDTHeader {
    fn new(base_ptr: u32) -> Self {
        let signature =
            unsafe { ptr::read_unaligned((base_ptr as *const [u8; 4]) as *const [u8; 4]) };
        let length = unsafe { ptr::read_unaligned((base_ptr as u64 + 4) as *const u32) };
        let revision = unsafe { ptr::read_unaligned((base_ptr as u64 + 8) as *const u8) };
        let checksum = unsafe { ptr::read_unaligned((base_ptr as u64 + 9) as *const u8) };
        let oem_id = unsafe { ptr::read_unaligned((base_ptr as u64 + 10) as *const [u8; 6]) };
        let oem_table_id = unsafe { ptr::read_unaligned((base_ptr as u64 + 16) as *const [u8; 8]) };
        let oem_revision = unsafe { ptr::read_unaligned((base_ptr as u64 + 24) as *const u32) };
        let creator_id = unsafe { ptr::read_unaligned((base_ptr as u64 + 28) as *const u32) };
        let creator_revision = unsafe { ptr::read_unaligned((base_ptr as u64 + 32) as *const u32) };
        ACPISDTHeader {
            signature,
            length,
            revision,
            checksum,
            oem_id,
            oem_table_id,
            oem_revision,
            creator_id,
            creator_revision,
        }
    }
    fn get_sum(&self) -> u32 {
        let mut sum: u32 = 0;
        let ptr = self as *const Self as *const u8;
        let size = core::mem::size_of::<Self>();
        for i in 0..size {
            let oct = unsafe { *ptr.offset(i as isize) };
            sum += oct as u32;
        }
        sum
    }
}
