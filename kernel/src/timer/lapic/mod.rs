pub fn lapic_calibrate() -> u32 {
    use super::pit::PIT;
    // use crate::drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
    let measure_duration: u32 = 10;
    let max_ticks = 0xFFFFFFFF;
    LAPICTimer::set_divide(LAPICTimerDivideValue::Div1);
    LAPICTimer::set_ticks(max_ticks);
    PIT::sleep(measure_duration as u64);
    LAPICTimer::set_active(false);
    let ticks_raw = max_ticks - LAPICTimer::get_current_ticks();
    let ticks = ticks_raw;
    // serial_println!("Ticks per 10ms: {}", ticks);
    LAPICTimer::set_lvt();
    LAPICTimer::set_periodic(true);
    LAPICTimer::set_divide(LAPICTimerDivideValue::Div1);
    LAPICTimer::set_ticks(ticks);
    LAPICTimer::set_active(false);
    ok!(
        "LAPICTimer calibrated with a frequency of {} MHz",
        ticks / 10000
    );
    return ticks;
}

fn lapic_calibrate_ticks() -> u32 {
    use super::pit::PIT;
    // use crate::drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
    let measure_duration: u32 = 10;
    let max_ticks = 0xFFFFFFFF;
    LAPICTimer::set_divide(LAPICTimerDivideValue::Div1);
    LAPICTimer::set_ticks(max_ticks);
    PIT::sleep(measure_duration as u64);
    LAPICTimer::set_active(false);
    let ticks_raw = max_ticks - LAPICTimer::get_current_ticks();
    let ticks = ticks_raw;
    // serial_println!("Ticks per 10ms: {}", ticks);
    LAPICTimer::set_lvt();
    LAPICTimer::set_periodic(true);
    LAPICTimer::set_divide(LAPICTimerDivideValue::Div1);
    LAPICTimer::set_ticks(ticks);
    LAPICTimer::set_active(false);
    return ticks / 1000; // Return ticks per 10 ms
}

#[derive(Debug, Copy, Clone)]
enum LAPICTimerDivideValue {
    Div2 = 0b0000,
    Div4 = 0b0001,
    Div8 = 0b0010,
    Div16 = 0b0011,
    Div32 = 0b1000,
    Div64 = 0b1001,
    Div128 = 0b1010,
    Div1 = 0b1011,
}
use crate::drivers::apic::local_apic::{LAPICReg, LOCAL_APIC};
use crate::interrupts::InterruptIndexAPIC;
use crate::{ok, serial_println};
// use crate::{print, serial_println};
pub struct LAPICTimer {}
impl LAPICTimer {
    fn set_lvt() {
        LOCAL_APIC.write_register(LAPICReg::TimerLVT, InterruptIndexAPIC::LAPICTimer.as_u32());
    }
    fn set_periodic(periodic: bool) {
        let mut lvt = LOCAL_APIC.read_register(LAPICReg::TimerLVT);
        if periodic {
            lvt = lvt | (1 << 17);
        } else {
            lvt = lvt & !(1 << 17);
        }
        LOCAL_APIC.write_register(LAPICReg::TimerLVT, lvt);
    }
    fn set_active(active: bool) {
        let mut lvt = LOCAL_APIC.read_register(LAPICReg::TimerLVT);
        if !active {
            lvt = lvt | (1 << 16);
        } else {
            lvt = lvt & !(1 << 16);
        }
        LOCAL_APIC.write_register(LAPICReg::TimerLVT, lvt);
    }
    fn set_divide(divide: LAPICTimerDivideValue) {
        let mut lvt = LOCAL_APIC.read_register(LAPICReg::TimerDCnf);
        lvt = lvt & 0b1111_1000;
        lvt = lvt | divide as u32;
        LOCAL_APIC.write_register(LAPICReg::TimerDCnf, lvt);
    }
    fn set_ticks(ticks: u32) {
        LOCAL_APIC.write_register(LAPICReg::TimerICnt, ticks);
    }
    fn get_ticks() -> u32 {
        LOCAL_APIC.read_register(LAPICReg::TimerCCnt)
    }
    fn get_current_ticks() -> u32 {
        LOCAL_APIC.read_register(LAPICReg::TimerCCnt)
    }
    // eroare 30ms
    pub fn sleep(millis: u64) {
        use crate::interrupts::handlers::{LAPIC_TIMER_SLEEP_COUNTER, LAPIC_TIMER_SLEEP_FLAG};
        use core::sync::atomic::Ordering;

        LAPIC_TIMER_SLEEP_COUNTER.store(millis, Ordering::Relaxed);
        LAPIC_TIMER_SLEEP_FLAG.store(true, Ordering::Relaxed);
        let ticks = lapic_calibrate_ticks();
        serial_println!("Sleeping for {} ticks", ticks);
        LAPICTimer::set_ticks(ticks);
        LAPICTimer::set_periodic(true);
        LAPICTimer::set_active(true);
        while LAPIC_TIMER_SLEEP_COUNTER.load(Ordering::Relaxed) > 0 {
            x86_64::instructions::hlt();
        }
        LAPIC_TIMER_SLEEP_FLAG.store(false, Ordering::Relaxed);
        LAPICTimer::set_active(false);
    }
    pub fn start_periodic_timer() {
        // use crate::interrupts::handlers::{LAPIC_TIMER_SLEEP_COUNTER, LAPIC_TIMER_SLEEP_FLAG};
        // use core::sync::atomic::Ordering;

        // LAPIC_TIMER_SLEEP_COUNTER.store(0, Ordering::Relaxed);
        // LAPIC_TIMER_SLEEP_FLAG.store(true, Ordering::Relaxed);
        let ticks = lapic_calibrate_ticks();
        LAPICTimer::set_ticks(ticks);
        LAPICTimer::set_periodic(true);
        LAPICTimer::set_active(true);
        serial_println!("LAPICTimer started with {} ticks per ms", ticks);
    }
}
