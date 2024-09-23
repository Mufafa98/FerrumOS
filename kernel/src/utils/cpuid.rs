use core::arch::asm;
pub fn check_apic() -> bool {
    let feat: i64;
    unsafe {
        asm!(
            "mov eax, 0x1",
            "cpuid",
            "mov {feat}, rdx",
            feat = out(reg) feat,
        );
    }
    feat & (1 << 9) != 0
}
