use crate::{print, println};

pub fn execute_command(command: &str) {
    match command.trim() {
        "help" => {
            println!("Available commands:");
            println!("  help - Show this help message");
            println!("  clear - Clear the screen");
            println!("  echo <message> - Echo the message back");
            println!("  ls <directory> - List files in the directory");
            println!("  touch <filename> - Create a new file with the given name");
            println!("  mkdir <dirname> - Create a new directory with the given name");
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H"); // ANSI escape code to clear the screen
        }
        cmd if cmd.starts_with("ls") => {
            let dir = cmd.trim_start_matches("ls");
            // Here you would implement the logic to list files in the directory
            // For now, we just print a placeholder message
            if dir.is_empty() {
                println!("No directory specified, listing current directory:");
                crate::fs::ext2::ls(None);
            }
            crate::fs::ext2::ls(Some(dir.trim()));
            // In a real implementation, you would read the directory and print its contents
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
        cmd if cmd.starts_with("echo ") => {
            let message = cmd.trim_start_matches("echo ");
            println!("{}", message);
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}
