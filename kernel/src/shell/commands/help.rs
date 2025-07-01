use crate::drivers::fonts::ansii_parser::ansii_builder::AnsiiString;
use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{colors, Command, Shell};
use alloc::string::String;
use alloc::vec::Vec;

use crate::alloc::string::ToString;
use crate::println;

pub struct HelpCommand {
    manual: ManualBuilder,
}

impl Command for HelpCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        HelpCommand {
            manual: ManualBuilder::new()
                .name("help")
                .short_description("Show available commands and their descriptions")
                .long_description(
                    "Displays a list of all available commands in the shell. \
                                   If a command is specified, it shows its manual.",
                )
                .usage("help [command [...]]")
                .arg("[command]", "The command(s) to show the manual for.")
                .example("help", "Show all available commands")
                .example("help ls", "Show the manual for the 'ls' command"),
        }
    }

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
                    // println!(
                    //     "{}{} {}",
                    //     cmd.name().to_string().fg(colors::CYAN) + ":",
                    //     " ".repeat(NAME_WIDTH - cmd.name().len()),
                    //     cmd.description()
                    // );
                    println!("{}", cmd.manual());
                } else {
                    println!("Command not found: {}", arg);
                }
            }
        }
    }

    fn description(&self) -> String {
        self.manual.build_short()
    }

    fn name(&self) -> &str {
        "help"
    }

    fn manual(&self) -> String {
        self.manual.build_long()
    }
}
