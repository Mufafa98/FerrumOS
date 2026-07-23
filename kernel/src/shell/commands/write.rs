use crate::print;
use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use alloc::string::String;
use alloc::vec::Vec;

pub struct WriteCommand {
    manual: ManualBuilder,
}
impl Command for WriteCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        WriteCommand {
            manual: ManualBuilder::new()
                .name("write")
                .short_description("Write content to a file")
                .long_description(
                    "Writes the specified content to the given file. \
            If the file does not exist, it will be created. If the file exists, \
            its content will be replaced unless the append option is used.",
                )
                .usage("write <file> [options] <content>")
                .arg("<file>", "The file to write to")
                .arg("<content>", "The content to write to the file")
                .arg(
                    "-a, --append",
                    "[options] Append content to the file instead of overwriting it",
                )
                .arg(
                    "-c, --create",
                    "[options] Create the file if it does not exist",
                )
                .example(
                    "write my_file.txt 'Hello, World!'",
                    "Writes 'Hello, World!' to 'my_file.txt'",
                )
                .example(
                    "write my_file.txt -a 'Hello again!'",
                    "Appends 'Hello again!' to 'my_file.txt'",
                ),
        }
    }
    fn execute(&self, args: Vec<&str>, _: &Shell) {
        if args.is_empty() || args.len() < 2 {
            print!("{}", self.manual.build_usage());
            return;
        }
        let file_path = args[0];
        let create = args.contains(&"-c") || args.contains(&"--create");
        let append = args.contains(&"-a") || args.contains(&"--append");

        use crate::fs::ext2::file::File;

        let mut file = match File::from_path(file_path) {
            Ok(file) => file,
            Err(_) if create => {
                use crate::fs::ext2::touch;
                touch(file_path);

                match File::from_path(file_path) {
                    Ok(file) => file,
                    Err(_) => {
                        // println!("Error creating file: {}", file_path);
                        return;
                    }
                }
            }
            Err(_) => {
                // println!("Error opening file: {}", file_path);
                return;
            }
        };

        let content = {
            if append {
                file.seek_end();
            }
            args[1..]
                .iter()
                .copied()
                .filter(|arg| {
                    *arg != "-a" && *arg != "--append" && *arg != "-c" && *arg != "--create"
                })
                .collect::<Vec<_>>()
                .join(" ")
        };

        if content.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }

        let mut offset = 0;
        loop {
            let bytes_to_write = content.len() - offset;
            if bytes_to_write == 0 {
                break;
            }
            let bytes_written = file.write(&content.as_bytes()[offset..], bytes_to_write);
            if bytes_written == 0 {
                // println!("Failed to write to file: {}", file_path);
                return;
            }
            offset += bytes_written;
        }

        // should add an argument for verbosity

        // if offset > 0 {
        //     println!("Successfully wrote {} bytes to {}", offset, file_path);
        // } else {
        //     println!("No content written to {}", file_path);
        // }
    }
    fn description(&self) -> String {
        self.manual.build_short()
    }
    fn name(&self) -> &str {
        "write"
    }
    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: write <file> [options] content>\n\n\
        // Writes the specified content to the given file.\n\n\
        // options:\n\
        // -a, --append    Append content to the file instead of overwriting it\n\
        // Example: write my_file.txt 'Hello, World!'\n\n\
        // This will create or overwrite 'my_file.txt' with the content 'Hello, World!'.\n\n\
        // Note: If the file already exists, its content will be replaced."
        //     .to_string()
    }
}
