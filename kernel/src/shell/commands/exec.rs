use crate::shell::{Command, Shell};

use crate::task::executor::{self, Executor};
use crate::task::Task;
use crate::{println, serial_print, serial_println, task};

pub struct ExecCommand;

impl Command for ExecCommand {
    fn description(&self) -> &str {
        "Executes a command in the shell"
    }

    fn execute(&self, args: alloc::vec::Vec<&str>, shell: &crate::shell::Shell) {
        if args.is_empty() {
            println!("Usage: exec [options]");
            return;
        }

        let option = args[0];

        match option {
            "-a" | "--add" => {
                let task_id = task::TaskId::new();
                let temp_task = Task::new_with_id(
                    task_id.as_u64(),
                    crate::task::demo_task::demo_task_runner('!', 1000, task_id),
                );
                use crate::task::ADD_TASK_Q;
                let mut add_task_q = ADD_TASK_Q.lock();
                if add_task_q.push(temp_task).is_err() {
                    println!("Failed to add task to the queue");
                } else {
                    println!("Task {:?} added to the queue", task_id);
                }
            }
            "-l" | "--list" => {
                todo!("Listing all programs is not implemented yet.");
            }
            _ => {
                println!("Unknown option: {}", option);
                println!("{}", self.manual());
            }
        }
    }

    fn manual(&self) -> &str {
        "Usage: exec [options]\n\
        Options:\n\
        -a, --add <program_name> [<args>...]  Add a new program to the shell\n\
        -l, --list                            List available programs\n\
        "
    }

    fn name(&self) -> &str {
        "exec"
    }
}
