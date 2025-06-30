use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::vec::Vec;
pub struct MkdirCommand;
impl Command for MkdirCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            println!("Usage: mkdir <directory_name1> [<directory_name2> ...]");
            return;
        }
        use crate::fs::ext2::mkdir;
        for dir_name in args {
            mkdir(dir_name);
        }
    }
    fn description(&self) -> &str {
        "Create a new directory"
    }
    fn name(&self) -> &str {
        "mkdir"
    }
    fn manual(&self) -> &str {
        "Usage: mkdir <directory_name> [<directory_name2> ...]\n\n\
        Creates a new directory with the specified name.\n\n\
        Example: mkdir new_folder\n\n\
        This will create a directory named 'new_folder'."
    }
}
