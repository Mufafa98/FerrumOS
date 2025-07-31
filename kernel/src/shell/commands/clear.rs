use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub struct ClearCommand {
    manual: ManualBuilder,
}

impl Command for ClearCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        ClearCommand {
            manual: ManualBuilder::new()
                .short_description("Clear the terminal screen")
                .long_description("Clears the terminal screen, removing all previous output.")
                .usage("clear")
                .example("clear", "Clears the terminal screen"),
        }
    }

    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        // Clear the terminal screen
        print!("\x1B[2J\x1B[1;1H");
    }

    fn description(&self) -> String {
        self.manual.build_short()
    }

    fn name(&self) -> &str {
        "clear"
    }

    fn manual(&self) -> String {
        self.manual.build_long()
    }
}
