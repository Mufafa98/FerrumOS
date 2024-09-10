use core::arch::asm;
pub fn inb(port: u16) -> u8 {
    let data: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") data,
        );
    }
    data
}
pub fn outb(port: u16, data: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") data,
        );
    }
}
