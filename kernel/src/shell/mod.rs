use crate::{
    drivers::fonts::{ansii_parser::ansii_builder::AnsiiString, color::colors},
    fs::ext2::file::File,
    print, println,
    shell::commands::write::WriteCommand,
};

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{boxed::Box, vec};
use alloc::{collections::btree_map::BTreeMap, format};
use lazy_static::lazy_static;
use spin::Mutex;

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

    history_cursor: Option<usize>,
    history: Vec<String>,
}
impl Default for Shell {
    fn default() -> Self {
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
        let mut shell = Shell {
            commands,
            key_buffer: String::new(),
            history_cursor: None,
            history: Vec::new(),
        };
        shell.load_from_file();
        shell
    }
}
impl Shell {
    pub fn handle_input(&mut self, key: pc_keyboard::DecodedKey) {
        use pc_keyboard::{DecodedKey, KeyCode};
        match key {
            DecodedKey::Unicode(c) => self.handle_unicode_input(c),
            DecodedKey::RawKey(KeyCode::ArrowUp) => self.show_previous_history(),
            DecodedKey::RawKey(KeyCode::ArrowDown) => self.show_next_history(),
            _ => {}
        }
    }

    fn handle_unicode_input(&mut self, c: char) {
        match c {
            '\n' => {
                if !self.key_buffer.is_empty() {
                    println!();
                    self.execute_command();
                    self.key_buffer.clear();
                    self.history_cursor = None;
                }
                print_caret();
            }
            '\x08' | '\x7f' => {
                if self.key_buffer.pop().is_some() {
                    print!("\x08 \x08");
                }
                self.history_cursor = None;
            }
            _ => {
                self.key_buffer.push(c);
                print!("{}", c);
                self.history_cursor = None;
            }
        }
    }

    fn clear_key_buffer(&mut self) {
        while self.key_buffer.pop().is_some() {
            print!("\x08 \x08");
        }
    }

    fn show_history_entry(&mut self, idx: usize) {
        if let Some(entry) = self.history.get(idx).cloned() {
            self.clear_key_buffer();
            self.key_buffer.push_str(&entry);
            print!("{}", entry);
        }
    }

    fn show_previous_history(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let next_cursor = match self.history_cursor {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(idx) => idx - 1,
        };

        self.history_cursor = Some(next_cursor);
        self.show_history_entry(next_cursor);
    }

    fn show_next_history(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_cursor {
            None => {}
            Some(idx) if idx + 1 < self.history.len() => {
                let next_cursor = idx + 1;
                self.history_cursor = Some(next_cursor);
                self.show_history_entry(next_cursor);
            }
            Some(_) => {
                self.history_cursor = None;
                self.clear_key_buffer();
            }
        }
    }

    fn load_from_file(&mut self) {
        let mut file = match File::from_path("cmd_history") {
            Ok(file) => file,
            Err(_) => return,
        };

        let mut buffer = [0u8; 1024];
        let mut command_buffer = String::new();

        loop {
            let bytes_read = file.read(&mut buffer, 1024);
            if bytes_read == 0 {
                break;
            }

            for item in buffer.iter().take(bytes_read) {
                if *item != 0 {
                    command_buffer.push(*item as char);
                }
            }
        }

        self.history = command_buffer
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
    }

    fn add_to_history(&mut self, command: &str, args: Vec<&str>) {
        let writer = WriteCommand::new();
        let cmd = format!("{} {}", command, args.join(" "));
        writer.execute(vec!["cmd_history", "-c", "-a", &cmd, "\n"], &self);
        self.history.push(cmd);
        self.history_cursor = None;
    }

    pub fn execute_command(&mut self) {
        let key_buffer = self.key_buffer.clone();
        let command_line = key_buffer.trim();

        if command_line.is_empty() {
            return;
        }

        let mut parts: Vec<_> = command_line.split_whitespace().collect();
        let command = parts.remove(0);
        let args: Vec<&str> = parts;

        self.add_to_history(command, args.clone());

        if let Some(cmd) = self.commands.get(command) {
            cmd.execute(args, self);
        } else {
            println!("Command not found: {}", command);
        }
    }

    pub fn list_commands(&self) {
        for cmd in self.commands.values() {
            println!(
                "{}: {}",
                cmd.name().to_string().fg(colors::CYAN),
                cmd.description()
            );
        }
    }

    fn get_commands(&self) -> &BTreeMap<String, Box<dyn Command>> {
        &self.commands
    }
}

// lazy_static!(
//     /// Last command
//     pub static ref SHELL: Shell = Shell::new();
// );

lazy_static! {
    /// Global shell instance
    pub static ref SHELL: Mutex<Shell> = Mutex::new(Shell::default());
}

pub fn print_caret() {
    print!("\x1B[s\r\x1B[1D{}\x1B[u", ">".fg(colors::LIGHT_CYAN));
}
