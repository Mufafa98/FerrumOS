use crate::shell::Command;
use crate::task::REM_TASK_Q;
use crate::{println, serial_println};

pub struct KillCommand;

impl Command for KillCommand {
    fn description(&self) -> &str {
        "Kills a task by its ID"
    }

    fn execute(&self, args: alloc::vec::Vec<&str>, _shell: &crate::shell::Shell) {
        if args.len() != 1 {
            println!("Usage: kill <task_id>");
            return;
        }

        let task_id = match args[0].parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                println!("Invalid task ID: {}", args[0]);
                return;
            }
        };
        {
            let mut rem_task_q = REM_TASK_Q.lock();
            rem_task_q.push(task_id);
            serial_println!("Task with ID {} has been scheduled for removal.", task_id);
        }
    }

    fn manual(&self) -> &str {
        "Usage: kill <task_id>\n\
         Kills the task with the specified ID."
    }

    fn name(&self) -> &str {
        "kill"
    }
}
