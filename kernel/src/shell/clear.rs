use crate::shell::Command;
use crate::{print, println};
use alloc::vec::Vec;
pub struct ClearCommand;

impl Command for ClearCommand {
    fn execute(&self, args: Vec<&str>) {
        // Clear the terminal screen
        print!("\x1B[2J\x1B[1;1H");
    }

    fn description(&self) -> &str {
        "Clear the terminal screen"
    }

    fn name(&self) -> &str {
        "clear"
    }

    fn manual(&self) -> &str {
        "Usage: clear\n\nClears the terminal screen."
    }
}
