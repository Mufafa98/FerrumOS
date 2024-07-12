#[allow(unused_imports)]
use super::{exit_qemu, QemuExitCode};
use core::panic::PanicInfo;
#[allow(unused_imports)]
use drivers::println;
use io::serial_println;

// This function is called on panic.
#[cfg(not(any(test, feature = "testing")))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// our panic handler in test mode
#[cfg(any(test, feature = "testing"))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
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
