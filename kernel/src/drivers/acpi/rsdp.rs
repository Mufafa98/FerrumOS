static RSDP_REQUEST: limine::request::RsdpRequest = limine::request::RsdpRequest::new();
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    // length: u32,
    // xsdt_address: u64,
    // extended_checksum: u8,
    // reserved: [u8; 3],
}
impl Rsdp {
    pub fn new() -> Self {
        let rsdp_response = RSDP_REQUEST.get_response().unwrap();
        let rsdp_address = rsdp_response.address();
        let signature = unsafe { *(rsdp_address as *const [u8; 8]) };
        let checksum = unsafe { *((rsdp_address as u64 + 8) as *const u8) };
        let oem_id = unsafe { *((rsdp_address as u64 + 9) as *const [u8; 6]) };
        let revision = unsafe { *((rsdp_address as u64 + 15) as *const u8) };
        let rsdt_address = unsafe { *((rsdp_address as u64 + 16) as *const u32) };
        Rsdp {
            signature,
            checksum,
            oem_id,
            revision,
            rsdt_address,
        }
    }
    pub fn is_valid(&self) -> bool {
        let mut sum: u32 = 0;
        let ptr = self as *const Self as *const u8;
        let size = core::mem::size_of::<Self>();
        for i in 0..size {
            let oct = unsafe { *ptr.offset(i as isize) };
            sum += oct as u32;
        }
        sum &= 0xff;
        sum == 0
    }
    pub fn rsdt_address(&self) -> u32 {
        self.rsdt_address
    }
}
