use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub struct LsCommand {
    manual: ManualBuilder,
}
impl Command for LsCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        LsCommand {
            manual: ManualBuilder::new()
                .name("ls")
                .short_description("List files in a directory")
                .long_description("Lists all files in the specified directory.")
                .usage("ls <directory>")
                .arg("<directory>", "The directory to list files from")
                .example(
                    "ls /home/user",
                    "Lists all files in the /home/user directory",
                ),
        }
    }
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        use crate::fs::ext2::FileData;
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
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
    fn description(&self) -> String {
        // "List files in a directory".to_string()

        self.manual.build_short()
    }
    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: ls <directory>\n\nLists all files in the specified directory.".to_string()
    }
}
