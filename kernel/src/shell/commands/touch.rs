use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct TouchCommand {
    manual: ManualBuilder,
}
impl Command for TouchCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        TouchCommand {
            manual: ManualBuilder::new()
                .name("touch")
                .short_description("Create a new file")
                .long_description("Creates a new file with a specified name.")
                .usage("touch <file_name> [<file_name> ...]")
                .arg("<file_name>", "The name of the file to create or update")
                .example("touch new_file.txt", "Creates a file named 'new_file.txt'")
                .example(
                    "touch file1.txt file2.txt",
                    "Creates 'file1.txt' and 'file2.txt'",
                ),
        }
    }
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }
        use crate::fs::ext2::touch;
        for file_name in args {
            touch(file_name);
        }
    }
    fn description(&self) -> String {
        // "Create a new file or update the timestamp of an existing file".to_string()

        self.manual.build_short()
    }
    fn name(&self) -> &str {
        "touch"
    }
    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: touch <file_name> [<file_name> ...]\n\n\
        // Creates a new file with the specified name or updates the timestamp of an existing file.\n\n\
        // Example: touch new_file.txt\n\n\
        // This will create a file named 'new_file.txt' if it does not exist, or \
        // update its timestamp if it does.\n\nNote: If the file already exists, \
        // its content will not be modified.".to_string()
    }
}
