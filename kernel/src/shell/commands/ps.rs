use alloc::vec;

use crate::println;
use crate::shell::Command;
use crate::task::RUNNING_TASKS;

pub struct PsCommand;

impl Command for PsCommand {
    fn description(&self) -> &str {
        "Lists all running tasks"
    }

    fn execute(&self, _args: alloc::vec::Vec<&str>, _shell: &crate::shell::Shell) {
        use crate::task::TaskId;
        use alloc::collections::BTreeSet;
        use alloc::vec::Vec;
        let running_tasks = RUNNING_TASKS.lock();

        for task_id in running_tasks.iter() {
            // Convert TaskId to u64 for display
            let id: u64 = task_id.as_u64();
            println!("Task ID: {}", id);
        }
    }

    fn manual(&self) -> &str {
        "Usage: ps\n\
         Lists all currently running tasks."
    }

    fn name(&self) -> &str {
        "ps"
    }
}
