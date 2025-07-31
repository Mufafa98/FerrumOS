use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub struct EchoCommand {
    manual: ManualBuilder,
}
impl Command for EchoCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        EchoCommand {
            manual: ManualBuilder::new()
                .name("echo")
                .short_description("Echo the provided message to the terminal")
                .long_description("Prints the provided message to the terminal.")
                .usage("echo <message>")
                .arg("<message>", "The message to echo")
                .example(
                    "echo Hello, World!",
                    "Prints 'Hello, World!' to the terminal",
                ),
        }
    }

    fn execute(&self, args: Vec<&str>, shell: &Shell) {
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }

        // Join the arguments with a space and print them
        let message = args.join(" ");
        println!("{}", message);
    }

    fn description(&self) -> String {
        self.manual.build_short()
        // "Echo the provided message to the terminal".to_string()
    }

    fn name(&self) -> &str {
        "echo"
    }

    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: echo <message>\n\nPrints the provided message to the terminal.".to_string()
    }
}
