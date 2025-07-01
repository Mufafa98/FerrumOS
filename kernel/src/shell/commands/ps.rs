use alloc::vec;

use crate::println;
use crate::shell::manual_builder::ManualBuilder;
use crate::shell::Command;
use crate::task::RUNNING_TASKS;
use alloc::string::{String, ToString};

pub struct PsCommand {
    manual: ManualBuilder,
}

impl Command for PsCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        PsCommand {
            manual: ManualBuilder::new()
                .name("ps")
                .short_description("Lists all currently running tasks.")
                .long_description("Lists all currently running tasks in the system.")
                .usage("ps")
                .example("ps", "Lists all currently running tasks."),
        }
    }
    fn description(&self) -> String {
        self.manual.build_short()
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

    fn manual(&self) -> String {
        self.manual.build_long()
        // "Usage: ps\n\
        //  Lists all currently running tasks."
        //     .to_string()
    }

    fn name(&self) -> &str {
        "ps"
    }
}
