use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub struct MkdirCommand {
    manual: ManualBuilder,
}
impl Command for MkdirCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        MkdirCommand {
            manual: ManualBuilder::new()
                .name("mkdir")
                .short_description("Create a new directory")
                .long_description("Creates a new directory with the specified name.")
                .usage("mkdir <directory_name> [<directory_name> ...]")
                .arg("<directory_name>", "The name of the directory to create")
                .example("mkdir new_folder", "Creates a directory named 'new_folder'")
                .example(
                    "mkdir dir1 dir2",
                    "Creates two directories named 'dir1' and 'dir2'",
                ),
        }
    }
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }
        use crate::fs::ext2::mkdir;
        for dir_name in args {
            mkdir(dir_name);
        }
    }
    fn description(&self) -> String {
        // "Create a new directory".to_string()

        self.manual.build_short()
    }
    fn name(&self) -> &str {
        "mkdir"
    }
    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: mkdir <directory_name> [<directory_name2> ...]\n\n\
        // Creates a new directory with the specified name.\n\n\
        // Example: mkdir new_folder\n\n\
        // This will create a directory named 'new_folder'."
        //     .to_string()
    }
}
