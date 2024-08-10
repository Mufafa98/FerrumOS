//! Utils crate responsable with panic handlers, test utils
//! and vm management

pub mod panic_module;
mod qemu_utils;
mod test_utils;

/// Util function to exit qemu with an error code
pub use qemu_utils::exit_qemu;
/// Enum that provides Qemu exit codes
pub use qemu_utils::QemuExitCode;
/// Util function for running tests
pub use test_utils::test_runner;
