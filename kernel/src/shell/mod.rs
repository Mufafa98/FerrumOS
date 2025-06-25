use crate::{print, println};

pub mod ctw;

static HELP_MESSAGE: &str = "Below are presented available functions!\n\
help - Show this help message\n\
clear - Clear the screen\n\
echo <message> - Echo the message back\n\
ls <directory> - List files in the directory\n\
touch <filename> - Create a new file with the given name\n\
mkdir <dirname> - Create a new directory with the given name\n\
rm <filename> - Remove a file with the given name\n";

pub fn execute_command(command: &str) {
    match command.trim() {
        "help" => {
            println!("{}", HELP_MESSAGE);
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H"); // ANSI escape code to clear the screen
        }
        cmd if cmd.starts_with("ls") => {
            use crate::fs::ext2::ls;
            let dir = cmd.trim_start_matches("ls");
            if dir.is_empty() {
                ls(None);
            } else {
                ls(Some(dir.trim()));
            }
        }
        cmd if cmd.starts_with("touch") => {
            let filename = cmd.trim_start_matches("touch");
            if filename.is_empty() {
                println!("No filename specified. Usage: touch <filename>");
                return;
            }
            crate::fs::ext2::touch(filename);
        }
        cmd if cmd.starts_with("mkdir") => {
            let dirname = cmd.trim_start_matches("mkdir");
            if dirname.is_empty() {
                println!("No directory name specified. Usage: mkdir <dirname>");
                return;
            }
            crate::fs::ext2::mkdir(dirname);
        }
        cmd if cmd.starts_with("rm") => {
            let filename = cmd.trim_start_matches("rm");
            if filename.is_empty() {
                println!("No filename specified. Usage: rm <filename>");
                return;
            }
            crate::fs::ext2::rm(filename);
        }
        cmd if cmd.starts_with("echo ") => {
            let message = cmd.trim_start_matches("echo ");
            println!("{}", message);
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}
