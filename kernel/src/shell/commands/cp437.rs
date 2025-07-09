use crate::shell::{manual_builder::ManualBuilder, Command};
use crate::{print, println};
use alloc::format;
use alloc::string::String;
pub struct CP437Command {
    manual: ManualBuilder,
}

impl Command for CP437Command {
    fn new() -> Self
    where
        Self: Sized,
    {
        CP437Command {
            manual: ManualBuilder::new()
                .name("cp437")
                .short_description("Print available CP437 characters")
                .long_description("Displays the available CP437 characters in a grid format.")
                .usage("cp437")
                .example("cp437", "Displays the CP437 character set"),
        }
    }
    fn name(&self) -> &str {
        "cp437"
    }
    fn description(&self) -> alloc::string::String {
        self.manual.build_short()
    }
    fn manual(&self) -> alloc::string::String {
        self.manual.build_long()
    }
    fn execute(&self, args: alloc::vec::Vec<&str>, shell: &crate::shell::Shell) {
        use crate::drivers::fonts::text_writer::TEXT_WRITER;
        TEXT_WRITER.lock().prin_available_chars();
    }
}
