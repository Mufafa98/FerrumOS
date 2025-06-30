use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::vec::Vec;

pub struct TouchCommand;
impl Command for TouchCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            println!("Usage: touch <file_name>");
            return;
        }
        use crate::fs::ext2::touch;
        for file_name in args {
            touch(file_name);
        }
    }
    fn description(&self) -> &str {
        "Create a new file or update the timestamp of an existing file"
    }
    fn name(&self) -> &str {
        "touch"
    }
    fn manual(&self) -> &str {
        "Usage: touch <file_name> [<file_name> ...]\n\n\
        Creates a new file with the specified name or updates the timestamp of an existing file.\n\n\
        Example: touch new_file.txt\n\n\
        This will create a file named 'new_file.txt' if it does not exist, or \
        update its timestamp if it does.\n\nNote: If the file already exists, \
        its content will not be modified."
    }
}
