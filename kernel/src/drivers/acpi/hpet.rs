use super::ACPISDTHeader;
pub enum HPETRegisters {
    GeneralCapabilitiesID = 0x00,
    GeneralConfiguration = 0x10,
    MainCounterValue = 0xF0,
}

#[derive(Debug)]
pub struct HPET {
    header: ACPISDTHeader,
    event_timer_block_id: u32,
    reseved: u32,
    address: u64,
    id: u8,
    min_clock_tick: u16,
    page_protection: u8,
}

impl HPET {
    pub fn new(base_ptr: u32) -> Self {
        use core::ptr::read_unaligned;

        let header = ACPISDTHeader::new(base_ptr);
        let header_size = core::mem::size_of::<ACPISDTHeader>() as u32;
        let new_ptr = base_ptr + header_size;
        unsafe {
            let event_timer_block_id = read_unaligned(new_ptr as *const u32);
            let reseved = read_unaligned((new_ptr + 4) as *const u32);
            let address = read_unaligned((new_ptr + 8) as *const u64);
            let id = read_unaligned((new_ptr + 16) as *const u8);
            let min_clock_tick = read_unaligned((new_ptr + 17) as *const u16);
            let page_protection = read_unaligned((new_ptr + 19) as *const u8);
            HPET {
                header,
                event_timer_block_id,
                reseved,
                address,
                id,
                min_clock_tick,
                page_protection,
            }
        }
    }
    pub fn enable(&self) {
        let mut config = self.get_register(HPETRegisters::GeneralConfiguration);
        config = config | 1;
        self.set_register(HPETRegisters::GeneralConfiguration, config);
    }
    pub fn disable(&self) {
        let mut config = self.get_register(HPETRegisters::GeneralConfiguration);
        config = config & !1;
        self.set_register(HPETRegisters::GeneralConfiguration, config);
    }
    pub fn set_register(&self, register: HPETRegisters, value: u64) {
        let offset = register as u64;
        let ptr = self.address + offset;
        unsafe { core::ptr::write_unaligned(ptr as *mut u64, value) }
    }
    pub fn get_register(&self, register: HPETRegisters) -> u64 {
        let offset = register as u64;
        let ptr = self.address + offset;
        unsafe { core::ptr::read_unaligned(ptr as *const u64) }
    }
    pub fn get_timer_n_config(&self, n: u8) -> HPETTimerConfig {
        let offset = 0x100 + (n as u64) * 0x20;
        let ptr = self.address + offset;
        // unsafe { core::ptr::read_unaligned(ptr as *const u64) }
        HPETTimerConfig::new(ptr)
    }
    pub fn set_timer_n_comparator(&self, n: u8, value: u64) {
        let offset = 0x108 + (n as u64) * 0x20;
        let ptr = self.address + offset;
        unsafe { core::ptr::write_unaligned(ptr as *mut u64, value) }
    }
}
#[derive(Debug)]
pub struct HPETTimerConfig {
    base_address: u64,
}
impl HPETTimerConfig {
    pub fn new(base_address: u64) -> Self {
        HPETTimerConfig { base_address }
    }
    pub fn get_register(&self) -> u64 {
        unsafe { core::ptr::read_unaligned(self.base_address as *const u64) }
    }
    pub fn set_interrupt_idx(&self, idx: u8) {
        let idx_mask = !((0b11111 << 9) as u64);
        let idx = (idx as u64) << 9;
        let mut config = self.get_register();
        config = config & idx_mask;
        config = config | idx;
        unsafe { core::ptr::write_unaligned(self.base_address as *mut u64, config) }
    }
    pub fn enable_interrupt(&self) {
        let mut config = self.get_register();
        config |= 0b100;
        unsafe { core::ptr::write_unaligned(self.base_address as *mut u64, config) }
    }
    pub fn disable_interrupt(&self) {
        let mut config = self.get_register();
        config &= !0b100;
        unsafe { core::ptr::write_unaligned(self.base_address as *mut u64, config) }
    }
}
