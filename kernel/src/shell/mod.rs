use crate::{
    drivers::fonts::{ansii_parser::ansii_builder::AnsiiString, color::colors},
    print, println, serial_println,
};

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use alloc::boxed::Box;
use alloc::string::{String, ToString};

mod cat;
mod clear;
mod echo;
mod help;
mod ls;
mod mkdir;
mod rm;
mod touch;
mod write;

trait Command: Send + Sync {
    fn execute(&self, args: Vec<&str>);
    fn description(&self) -> &str;
    fn name(&self) -> &str;
    fn manual(&self) -> &str;
}

pub struct Shell {
    commands: BTreeMap<String, Box<dyn Command>>,
}
impl Shell {
    pub fn new() -> Self {
        let mut commands = BTreeMap::new();
        commands.insert(
            "clear".to_string(),
            Box::new(clear::ClearCommand) as Box<dyn Command>,
        );
        commands.insert(
            "help".to_string(),
            Box::new(help::HelpCommand) as Box<dyn Command>,
        );
        commands.insert(
            "echo".to_string(),
            Box::new(echo::EchoCommand) as Box<dyn Command>,
        );
        commands.insert(
            "cat".to_string(),
            Box::new(cat::CatCommand) as Box<dyn Command>,
        );
        commands.insert(
            "ls".to_string(),
            Box::new(ls::LsCommand) as Box<dyn Command>,
        );
        commands.insert(
            "mkdir".to_string(),
            Box::new(mkdir::MkdirCommand) as Box<dyn Command>,
        );
        commands.insert(
            "rm".to_string(),
            Box::new(rm::RmCommand) as Box<dyn Command>,
        );
        commands.insert(
            "touch".to_string(),
            Box::new(touch::TouchCommand) as Box<dyn Command>,
        );
        commands.insert(
            "write".to_string(),
            Box::new(write::WriteCommand) as Box<dyn Command>,
        );
        Shell { commands }
    }

    pub fn execute_command(&self, command: &str) {
        let command = command.trim();
        let mut parts: Vec<_> = command.split_whitespace().collect();
        let command = parts.remove(0);
        let args: Vec<&str> = parts;
        serial_println!("Executing command: {:?}", command);
        if let Some(cmd) = self.commands.get(command) {
            // cmd.execute(args.is_empty().then(|| None).unwrap_or(Some(&args)));
            cmd.execute(args);
        } else {
            println!("Command not found: {}", command);
        }
    }

    pub fn list_commands(&self) {
        for (name, cmd) in &self.commands {
            println!(
                "{}: {}",
                cmd.name().to_string().fg(colors::CYAN),
                cmd.description()
            );
        }
    }
}

lazy_static!(
    /// Last command
    pub static ref SHELL: Shell = Shell::new();
);

pub fn print_caret() {
    print!("\x1B[s\r\x1B[1D{}\x1B[u", ">".fg(colors::LIGHT_CYAN));
}
