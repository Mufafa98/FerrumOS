use super::Time;
use crate::drivers::acpi::hpet::{HPETRegisters, HPET};
pub struct HPETTimer {
    hpet: HPET,
    frequency: Time,
}
impl HPETTimer {
    pub fn new() -> Self {
        use crate::drivers::acpi::{rsdp::Rsdp, rsdt::RSDT};
        let rsdp = Rsdp::new();
        let rsdt_table = RSDT::new(rsdp.rsdt_address());
        let hpet = rsdt_table.get_hpet().unwrap();
        let frequency =
            Time::Femtoseconds(hpet.get_register(HPETRegisters::GeneralCapabilitiesID) >> 32);
        hpet.get_timer_n_config(2).set_interrupt_idx(0x12);
        hpet.get_timer_n_config(2).enable_interrupt();
        HPETTimer { hpet, frequency }
    }
    // TODO better class design
    pub fn sleep(&self, duration: Time) {
        use crate::interrupts::handlers::{HPET_SLEEP_COUNTER, HPET_SLEEP_FLAG};
        use core::sync::atomic::Ordering;
        let ticks = duration.to_nanoseconds() * self.frequency.to_nanoseconds();

        self.hpet.set_register(HPETRegisters::MainCounterValue, 0);
        self.hpet
            .set_timer_n_comparator(2, duration.to_nanoseconds());
        // HPET_SLEEP_COUNTER.store(ticks as i64, Ordering::Relaxed);
        let start = HPET_SLEEP_COUNTER.load(Ordering::Relaxed);

        // serial_println!("sleeping for {} ticks", ticks);
        self.hpet.enable();
        HPET_SLEEP_FLAG.store(true, Ordering::Relaxed);

        // serial_println!("start");
        while HPET_SLEEP_COUNTER.load(Ordering::Relaxed) - start < ticks.try_into().unwrap() {
            // NU
            self.hpet.set_register(HPETRegisters::MainCounterValue, 0);
            self.hpet
                .set_timer_n_comparator(2, duration.to_nanoseconds());
            x86_64::instructions::hlt();
        }
        // serial_println!("end");
        self.hpet.disable();
        HPET_SLEEP_FLAG.store(false, Ordering::Relaxed);
    }
}
