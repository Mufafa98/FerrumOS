use crate::drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
use crate::interrupts::handlers::{LAPIC_TIMER_SLEEP_COUNTER, LAPIC_TIMER_SLEEP_FLAG};
use crate::interrupts::InterruptIndexAPIC;
use crate::io::serial;
use crate::{serial_println, timer};
use core::sync::atomic::Ordering;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts;
fn lapic_ticks() -> u32 {
    use super::pit::PIT;
    use crate::drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
    let measure_duration: u32 = 1000;
    LOCAL_APIC.write_register(LAPICReg::TimerDCnf, 0x3);
    LOCAL_APIC.write_register(LAPICReg::TimerICnt, 0xFFFFFFFF);
    PIT::sleep(measure_duration as u64);
    // LOCAL_APIC.write_register(LAPICReg::TimerLVT, 1 << 16);
    let ticks_raw = 0xFFFFFFFF - LOCAL_APIC.read_register(LAPICReg::TimerCCnt);
    // serial_println!("Ticks {} {} {}", ticks_raw, 0xFFFFFFFFu32, measure_duration);
    ticks_raw / measure_duration
}
pub struct LAPICTimer {
    ticks_per_ms: u32,
}
impl LAPICTimer {
    pub fn init(&self) {
        // let lvt = 1 << 17 | InterruptIndexAPIC::LAPICTimer.as_u32();
        // LOCAL_APIC.write_register(LAPICReg::TimerLVT, lvt);
        // LOCAL_APIC.write_register(LAPICReg::TimerDCnf, 0x3);
    }
    fn set_lvt(&self) {
        LOCAL_APIC.write_register(LAPICReg::TimerLVT, InterruptIndexAPIC::LAPICTimer.as_u32());
    }
    fn set_periodic(&self, periodic: bool) {
        let mut lvt = LOCAL_APIC.read_register(LAPICReg::TimerLVT);
        if periodic {
            lvt = lvt | (1 << 17);
        } else {
            lvt = lvt & !(1 << 17);
        }
        LOCAL_APIC.write_register(LAPICReg::TimerLVT, lvt);
    }
    fn set_active(&self, active: bool) {
        let mut lvt = LOCAL_APIC.read_register(LAPICReg::TimerLVT);
        if active {
            lvt = lvt | (1 << 16);
        } else {
            lvt = lvt & !(1 << 16);
        }
        LOCAL_APIC.write_register(LAPICReg::TimerLVT, lvt);
    }
    pub fn sleep(&self, millis: u64) {
        LAPIC_TIMER_SLEEP_COUNTER.store(millis.try_into().unwrap(), Ordering::Relaxed);
        LAPIC_TIMER_SLEEP_FLAG.store(true, Ordering::Relaxed);
        // serial_println!("LTicks {}", lapic_ticks());
        self.set_lvt();
        self.set_periodic(true);
        LOCAL_APIC.write_register(LAPICReg::TimerDCnf, 0x0);
        self.set_active(false);
        LOCAL_APIC.write_register(LAPICReg::TimerICnt, 0x1);
        serial_println!(
            "Register : {:b}",
            LOCAL_APIC.read_register(LAPICReg::TimerICnt)
        );
        // LOCAL_APIC.write_register(LAPICReg::TimerLVT, 1 << 16);
        // while LAPIC_TIMER_SLEEP_COUNTER.load(Ordering::Relaxed) > 0 {
        //     serial_println!("{:?}", LAPIC_TIMER_SLEEP_FLAG.load(Ordering::Relaxed));
        //     x86_64::instructions::hlt();
        // }
        // LAPIC_TIMER_SLEEP_FLAG.store(false, Ordering::Relaxed);
    }
}

lazy_static! {
    pub static ref LAPIC_TIMER: LAPICTimer = LAPICTimer {
        ticks_per_ms: 1,
        // ticks_per_ms: lapic_ticks() / 10000,
    };
}
