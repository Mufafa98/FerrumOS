use crate::drivers::fonts::ansii_parser::ansii_builder::AnsiiString;
use crate::shell::{colors, Command, Shell};
use alloc::vec::Vec;

use crate::alloc::string::ToString;
use crate::println;

pub struct HelpCommand;

impl Command for HelpCommand {
    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        const NAME_WIDTH: usize = 6;
        if args.is_empty() {
            println!("Available commands:");
            for cmd in shell.get_commands().values() {
                println!(
                    "{}{} {}",
                    cmd.name().to_string().fg(colors::CYAN) + ":",
                    " ".repeat(NAME_WIDTH - cmd.name().len()),
                    cmd.description()
                );
            }
            println!("\nUse 'help <command>' to see more details about a specific command.");
        } else {
            for arg in args {
                if let Some(cmd) = shell.get_commands().get(arg) {
                    println!(
                        "{}{} {}",
                        cmd.name().to_string().fg(colors::CYAN) + ":",
                        " ".repeat(NAME_WIDTH - cmd.name().len()),
                        cmd.description()
                    );
                    println!("{}", cmd.manual());
                } else {
                    println!("Command not found: {}", arg);
                }
            }
        }
    }

    fn description(&self) -> &str {
        "Show available commands and their descriptions"
    }

    fn name(&self) -> &str {
        "help"
    }

    fn manual(&self) -> &str {
        "Usage: help [command]\n\n\
         Lists all available commands.\n\
         If a command is specified, shows its manual.\n\n\
         Example: help clear\n\n\
         This will display the manual for the 'clear' command."
    }
}
