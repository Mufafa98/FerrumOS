use crate::shell::Command;
use crate::{print, println};
use alloc::vec::Vec;
pub struct EchoCommand;
impl Command for EchoCommand {
    fn execute(&self, args: Vec<&str>) {
        if args.is_empty() {
            println!("Usage: echo <message>");
            return;
        }

        // Join the arguments with a space and print them
        let message = args.join(" ");
        println!("{}", message);
    }

    fn description(&self) -> &str {
        "Echo the provided message to the terminal"
    }

    fn name(&self) -> &str {
        "echo"
    }

    fn manual(&self) -> &str {
        "Usage: echo <message>\n\nPrints the provided message to the terminal."
    }
}
