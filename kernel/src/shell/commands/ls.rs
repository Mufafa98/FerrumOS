use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::vec::Vec;
pub struct LsCommand;
impl Command for LsCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        use crate::fs::ext2::FileData;
        if args.is_empty() {
            println!("Usage: ls <directory>");
            return;
        }

        let result = crate::fs::ext2::ls(Some(args[0]));
        println!(
            "{:<8} {:<5} {:<20} {:>10} bytes",
            "Inode", "Type", "Name", "Size"
        );
        for file in result.iter() {
            println!(
                "{:<8} {:<5} {:<20} {:>10} bytes",
                file.inode, file.entry_type, file.name, file.size
            );
        }
    }
    fn name(&self) -> &str {
        "ls"
    }
    fn description(&self) -> &str {
        "List files in a directory"
    }
    fn manual(&self) -> &str {
        "Usage: ls <directory>\n\nLists all files in the specified directory."
    }
}
