use crate::shell::manual_builder::ManualBuilder;
use crate::shell::Command;
use crate::task::REM_TASK_Q;
use crate::{print, println, serial_println};
use alloc::string::{String, ToString};

pub struct KillCommand {
    manual: ManualBuilder,
}

impl Command for KillCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        KillCommand {
            manual: ManualBuilder::new()
            .name("kill")
            .short_description("Kills a task by its ID")
            .long_description("Kills a task by its ID. This command is used to terminate a running task in the system.")
            .usage("kill <task_id>")
            .arg("<task_id>", "The ID of the task to be killed")
            .example("kill 12345", "Kills the task with ID 12345"),
        }
    }
    fn description(&self) -> String {
        // "Kills a task by its ID".to_string()
        self.manual.build_short()
    }

    fn execute(&self, args: alloc::vec::Vec<&str>, _shell: &crate::shell::Shell) {
        if args.len() != 1 {
            print!("{}", self.manual.build_usage());
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

    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: kill <task_id>\n\
        //  Kills the task with the specified ID."
        //     .to_string()
    }

    fn name(&self) -> &str {
        "kill"
    }
}
