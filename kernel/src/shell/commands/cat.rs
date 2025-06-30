use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::vec::Vec;
pub struct CatCommand;
impl Command for CatCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            println!("Usage: cat <file>");
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

    fn description(&self) -> &str {
        "Concatenate and display file contents"
    }

    fn name(&self) -> &str {
        "cat"
    }

    fn manual(&self) -> &str {
        "Usage: cat <file> [<file> ...]\n\n\
        Displays the contents of the specified file(s)."
    }
}
