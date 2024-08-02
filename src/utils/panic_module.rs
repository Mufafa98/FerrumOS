#[allow(unused_imports)]
use super::{exit_qemu, QemuExitCode};
#[allow(unused_imports)]
use crate::{print, println, serial_print, serial_println};
#[allow(unused_imports)]
use core::panic::PanicInfo;

// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    crate::hlt_loop();
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    crate::hlt_loop();
}

// #[panic_handler]
// pub fn panic(info: &PanicInfo) -> ! {
//     serial_println!("{}", cfg!(test));
//     if cfg!(test) {
//         serial_println!("[failed]\n");
//         serial_println!("Error: {}\n", info);
//         exit_qemu(QemuExitCode::Failed);
//         loop {}
//     } else {
//         println!("{}", info);
//         loop {}
//     }
// }
