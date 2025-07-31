pub mod pit_config;
use pit_config::*;

use crate::{
    // print, serial_print, serial_println,
    // timer::pit,
    utils::registers::{inb, outb},
};
const BASE_FREQUENCY: f32 = 3579545.0 / 3.0;
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
        self.reload = (BASE_FREQUENCY / 1000.0 * millis as f32) as u16;
    }
    pub fn set_encoding(&mut self, encoding: PITEncoding) {
        self.config.set_encoding(encoding);
    }
    pub fn set_mode(&mut self, mode: PITOperatingMode) {
        self.config.set_mode(mode);
    }
    pub fn set_access_mode(&mut self, access_mode: PITAccessMode) {
        self.config.set_access_mode(access_mode);
    }
    pub fn set_channel(&mut self, channel: PITChannel) {
        self.config.set_channel(channel);
    }
    pub fn start(&self) {
        use crate::utils::registers::outb;
        x86_64::instructions::interrupts::disable();
        outb(0x42, self.config.get_config());
        outb(0x40, (self.reload & 0xFF) as u8);
        inb(0x60); // give a little delay
        outb(0x40, (self.reload >> 8) as u8);
        x86_64::instructions::interrupts::enable();
    }
    // Error of 0.01ms
    pub fn sleep(millis: u64) {
        use crate::interrupts::handlers::{PIT_SLEEP_COUNTER, PIT_SLEEP_FLAG};
        use core::sync::atomic::Ordering;
        {
            const DIVISOR: u32 = (BASE_FREQUENCY / 1000.0) as u32;
            outb(0x43, 0b00110100);
            outb(0x40, (DIVISOR & 0xFF).try_into().unwrap());
            outb(0x40, (DIVISOR >> 8) as u8);
        }
        PIT_SLEEP_COUNTER.store(millis.try_into().unwrap(), Ordering::Relaxed);
        PIT_SLEEP_FLAG.store(true, Ordering::Relaxed);
        while PIT_SLEEP_COUNTER.load(Ordering::Relaxed) > 0 {
            x86_64::instructions::hlt();
        }
        PIT_SLEEP_FLAG.store(false, Ordering::Relaxed);
    }
    pub fn get_counter() -> u64 {
        use crate::interrupts::handlers::PIT_COUNTER;
        let mut timer = PIT::new();
        timer.set_timer(1);
        timer.set_mode(PITOperatingMode::RateGenerator);
        timer.start();
        PIT_COUNTER.load(core::sync::atomic::Ordering::Relaxed)
    }
}
