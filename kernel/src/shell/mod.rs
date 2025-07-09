use crate::{
    drivers::fonts::{ansii_parser::ansii_builder::AnsiiString, color::colors},
    print, println, serial_println,
};

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use alloc::boxed::Box;
use alloc::string::{String, ToString};

pub mod commands;
pub mod input_dispatcher;
pub mod manual_builder;
use commands::*;

trait Command: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn execute(&self, args: Vec<&str>, shell: &Shell);
    fn description(&self) -> String;
    fn name(&self) -> &str;
    fn manual(&self) -> String;
}

macro_rules! add_commands {
    ($commands:ident, $($module:ident => $struct:ident),* $(,)?) => {
        $(
            $commands.insert(
                stringify!($module).to_string(),
                Box::new($module::$struct::new()) as Box<dyn Command>,
            );
        )*
    };
}

pub struct Shell {
    commands: BTreeMap<String, Box<dyn Command>>,
    key_buffer: String,
}
impl Shell {
    pub fn new() -> Self {
        let mut commands = BTreeMap::new();
        add_commands!(commands,
            help => HelpCommand,        // HELP
            clear => ClearCommand,      // SCRN
            echo => EchoCommand,        // SCRN
            cat => CatCommand,          // FLST
            ls => LsCommand,            // FLST
            mkdir => MkdirCommand,      // FLST
            rm => RmCommand,            // FLST
            touch => TouchCommand,      // FLST
            write => WriteCommand,      // FLST
            exec => ExecCommand,        // ASAW
            kill => KillCommand,        // ASAW
            ps => PsCommand,            // ASAW
            // TODO: Add command
            // cp437 => CP437Command,
        );
        Shell {
            commands,
            key_buffer: String::new(),
        }
    }

    pub fn handle_input(&mut self, key: pc_keyboard::DecodedKey) {
        match key {
            pc_keyboard::DecodedKey::Unicode(c) => {
                if c == '\n' {
                    println!();
                    self.execute_command(&self.key_buffer);
                    self.key_buffer.clear();
                    print_caret();
                } else if c == '\x08' || c == '\x7f' {
                    self.key_buffer.pop();
                    print!("\x08 \x08");
                } else {
                    self.key_buffer.push(c);
                    print!("{}", c);
                }
            }
            _ => {
                // println!("Unsupported key: {:?}", key);
            }
        }
    }

    pub fn execute_command(&self, command: &str) {
        let command = command.trim();
        let mut parts: Vec<_> = command.split_whitespace().collect();
        let command = parts.remove(0);
        let args: Vec<&str> = parts;
        serial_println!("Executing command: {:?}", command);
        if let Some(cmd) = self.commands.get(command) {
            cmd.execute(args, self);
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

    pub fn get_commands(&self) -> &BTreeMap<String, Box<dyn Command>> {
        &self.commands
    }
}

// lazy_static!(
//     /// Last command
//     pub static ref SHELL: Shell = Shell::new();
// );

lazy_static! {
    /// Global shell instance
    pub static ref SHELL: Mutex<Shell> = Mutex::new(Shell::new());
}

pub fn print_caret() {
    print!("\x1B[s\r\x1B[1D{}\x1B[u", ">".fg(colors::LIGHT_CYAN));
}
