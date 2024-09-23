use core::arch::asm;
pub fn read_msr(msr: u32) -> u64 {
    unsafe {
        let mut low: u32;
        let mut high: u32;
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
        ((high as u64) << 32) | (low as u64)
    }
}

pub fn write_msr(msr: u32, low: u32, high: u32) {
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
        );
    }
}
