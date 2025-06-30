use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::vec::Vec;
pub struct RmCommand;
impl Command for RmCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            println!("Usage: rm <file_or_directory>");
            return;
        }
        use crate::fs::ext2::rm;
        for path in args {
            rm(path);
        }
    }
    fn description(&self) -> &str {
        "Remove a file or directory"
    }
    fn name(&self) -> &str {
        "rm"
    }
    fn manual(&self) -> &str {
        "Usage: rm <file_or_directory> [<file_or_directory> ...]\n\n\
        Removes the specified file or directory.\n\n\
        Example: rm old_file.txt\n\n\
        This will remove the file named 'old_file.txt'.\n\n\
        Note: Use with caution, as this command will permanently delete files."
    }
}
