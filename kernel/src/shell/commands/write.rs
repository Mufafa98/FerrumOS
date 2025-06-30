use crate::shell::{Command, Shell};
use crate::{print, println, serial_println};
use alloc::vec::Vec;

pub struct WriteCommand;
impl Command for WriteCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            println!("Usage: write <file> [options] <content>");
            return;
        }
        if args.len() < 2 {
            println!("Usage: write <file> [options] <content>");
            return;
        }
        let file_path = args[0];
        use crate::fs::ext2::file::File;
        let mut file = File::from_path(file_path);
        if file.is_err() {
            println!("Error opening file: {}", file_path);
            return;
        }
        let mut file = file.unwrap();
        let mut content = args[1..].join(" ");
        if args[1] == "-a" || args[1] == "--append" {
            file.seek_end();
            content = args[2..].join(" ");
        }
        let mut buffer = [0u8; 1024];
        let mut offset = 0;
        loop {
            let bytes_to_write = content.as_bytes().len() - offset;
            if bytes_to_write <= 0 {
                break; // All content has been written
            }
            let bytes_written = file.write(&content.as_bytes()[offset..], bytes_to_write);
            if bytes_written == 0 {
                println!("Failed to write to file: {}", file_path);
                return;
            }
            offset += bytes_written;
        }
        if offset > 0 {
            println!("Successfully wrote {} bytes to {}", offset, file_path);
        } else {
            println!("No content written to {}", file_path);
        }
    }
    fn description(&self) -> &str {
        "Write content to a file"
    }
    fn name(&self) -> &str {
        "write"
    }
    fn manual(&self) -> &str {
        "Usage: write <file> [options] content>\n\n\
        Writes the specified content to the given file.\n\n\
        options:\n\
        -a, --append    Append content to the file instead of overwriting it\n\
        Example: write my_file.txt 'Hello, World!'\n\n\
        This will create or overwrite 'my_file.txt' with the content 'Hello, World!'.\n\n\
        Note: If the file already exists, its content will be replaced."
    }
}
