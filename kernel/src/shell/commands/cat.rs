use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub struct CatCommand {
    manual: ManualBuilder,
}
impl Command for CatCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        CatCommand{
            manual: ManualBuilder::new()
                .name("cat")
                .short_description("Concatenate and display file contents")
                .long_description(
                    "Concatenates the contents of one or more files and displays them on the standard output. \
                     If a file is not found, an error message is printed.",
                )
                .usage("cat <file> [<file> ...]")
                .arg("<file>", "The file(s) to display.")
                .example("cat file.txt", "Display the contents of 'file.txt'.")
                .example("cat file1.txt file2.txt", "Display the contents of 'file1.txt' and 'file2.txt'."),
        }
    }

    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }
        use crate::fs::ext2::file::File;
        let mut args = args;
        for file_path in args {
            let mut file = File::from_path(file_path);
            if file.is_err() {
                println!("Error opening file: {}", file_path);
                continue;
            }
            let mut file = file.unwrap();
            let mut buffer = [0u8; 1024];
            loop {
                let bytes_read = file.read(&mut buffer, 1024);
                if bytes_read == 0 {
                    break; // End of file
                }
                for i in 0..bytes_read {
                    if buffer[i] == 0 {
                        continue; // Skip null bytes
                    }
                    print!("{}", buffer[i] as char);
                }
            }
        }
        println!(); // Ensure a newline at the end
    }

    fn description(&self) -> String {
        self.manual.build_short()
    }

    fn name(&self) -> &str {
        "cat"
    }

    fn manual(&self) -> String {
        self.manual.build_long()
    }
}
