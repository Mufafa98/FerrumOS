use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub struct RmCommand {
    manual: ManualBuilder,
}
impl Command for RmCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        RmCommand {
            manual: ManualBuilder::new()
                .name("rm")
                .short_description("Remove a file or directory")
                .long_description("Removes the specified file or directory.")
                .usage("rm <file_or_directory> [<file_or_directory> ...]")
                .arg("<file_or_directory>", "The file or directory to remove")
                .example("rm old_file.txt", "Removes the file named 'old_file.txt'")
                .example("rm old_folder", "Removes the directory named 'old_folder'"),
        }
    }
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }
        use crate::fs::ext2::rm;
        for path in args {
            rm(path);
        }
    }
    fn description(&self) -> String {
        self.manual.build_short()
        // "Remove a file or directory".to_string()
    }
    fn name(&self) -> &str {
        "rm"
    }
    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: rm <file_or_directory> [<file_or_directory> ...]\n\n\
        // Removes the specified file or directory.\n\n\
        // Example: rm old_file.txt\n\n\
        // This will remove the file named 'old_file.txt'.\n\n\
        // Note: Use with caution, as this command will permanently delete files."
        //     .to_string()
    }
}
