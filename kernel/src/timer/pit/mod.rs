mod pit_config;
use pit_config::*;
const BASE_FREQUENCY: u32 = 1193182;
pub struct PIT {
    config: PITConfig,
    reload: u16,
}
impl PIT {
    pub fn new() -> Self {
        PIT {
            config: PITConfig::build_from(
                PITEncoding::Binary,
                PITOperatingMode::RateGenerator,
                PITAccessMode::AccessLowByteThenHighByte,
                PITChannel::Channel2,
            ),
            reload: 0,
        }
    }
    // 1ms
    pub fn set_timer(&mut self, millis: u16) {
        self.reload = (BASE_FREQUENCY as f32 / 1000.0 * millis as f32) as u16;
    }
    pub fn start(&self) {
        use crate::utils::registers::outb;
        outb(0x42, self.config.get_config());
        outb(0x40, (self.reload & 0xFF) as u8);
        outb(0x40, (self.reload >> 8) as u8);
    }
}
