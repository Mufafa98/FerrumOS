#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod panic_module;
mod qemu_utils;
mod test_utils;

pub use qemu_utils::exit_qemu;
pub use qemu_utils::QemuExitCode;

pub use test_utils::test_runner_function;
