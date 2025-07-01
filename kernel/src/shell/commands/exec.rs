use crate::shell::manual_builder::ManualBuilder;
use crate::shell::{Command, Shell};
use crate::task::executor::{self, Executor};
use crate::task::Task;
use crate::{println, serial_print, serial_println, task};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct ExecCommand {
    manual: ManualBuilder,
}

impl Command for ExecCommand {
    fn new() -> Self
    where
        Self: Sized,
    {
        ExecCommand {
            manual: ManualBuilder::new()
                .name("exec")
                .short_description("Executes one or more commands in the shell.")
                .long_description("Executes one or more commands in the shell.")
                .usage("exec [options]")
                .arg(
                    "-a, --add <program_name> [<args>...]",
                    "Adds a new program to the shell and executes it with <args>.",
                )
                .arg(
                    "-l, --list",
                    "Lists all available programs that can be executed.",
                )
                .example(
                    "exec -a async_demo a 100",
                    "Adds a new program named 'async_demo' with argument 'a' and value '100'.",
                )
                .example(
                    "exec -a async_demo a 100 & async_demo b 200",
                    "Adds two programs 'async_demo' with different arguments and \
                    runs them concurrently.",
                ),
        }
    }
    fn execute(&self, args: alloc::vec::Vec<&str>, shell: &crate::shell::Shell) {
        if args.is_empty() {
            print!("{}", self.manual.build_usage());
            return;
        }

        let option = args[0];

        match option {
            "-a" | "--add" => {
                if args.len() < 3 {
                    println!("Usage: exec -a <program_name> [<args>...]");
                    return;
                }
                fn run_async_demo(args: &[&str]) {
                    let new_args = &args[1..];
                    if new_args.len() < 2 {
                        println!("Usage: exec -a async_demo <char> <time to sleep>");
                        return;
                    }
                    let char_to_print = new_args[0].chars().next().unwrap_or(' ');
                    let sleep_time: u64 = match new_args[1].parse() {
                        Ok(time) => time,
                        Err(_) => {
                            println!("Invalid time to sleep: {}", new_args[1]);
                            return;
                        }
                    };
                    let task_id = task::TaskId::new();
                    let temp_task = Task::new_with_id(
                        task_id.as_u64(),
                        crate::task::demo_task::demo_task_runner(
                            char_to_print,
                            sleep_time,
                            task_id,
                        ),
                    );
                    use crate::task::ADD_TASK_Q;
                    let mut add_task_q = ADD_TASK_Q.lock();
                    if add_task_q.push(temp_task).is_err() {
                        println!("Failed to add task to the queue");
                    } else {
                        println!("Task {:?} added to the queue", task_id);
                    }
                }
                let new_args = &args[1..];
                if new_args.contains(&"&") {
                    let mut temp_args: Vec<&str> = Vec::new();
                    for arg in new_args {
                        if *arg == "&" {
                            if !temp_args.is_empty() {
                                let process = temp_args[0];
                                match process {
                                    "async_demo" => {
                                        run_async_demo(&temp_args);
                                    }
                                    _ => {
                                        println!("Program '{}' is not recognized.", process);
                                        println!("Use 'exec -l' to list available programs.");
                                    }
                                }
                                temp_args.clear();
                            }
                        } else if arg == new_args.last().unwrap() {
                            temp_args.push(arg);
                            if !temp_args.is_empty() {
                                let process = temp_args[0];
                                match process {
                                    "async_demo" => {
                                        run_async_demo(&temp_args);
                                    }
                                    _ => {
                                        println!("Program '{}' is not recognized.", process);
                                        println!("Use 'exec -l' to list available programs.");
                                    }
                                }
                                temp_args.clear();
                            }
                        } else {
                            temp_args.push(arg);
                        }
                    }
                } else {
                    let process = new_args[0];
                    match process {
                        "async_demo" => {
                            run_async_demo(new_args);
                        }
                        _ => {
                            println!("Program '{}' is not recognized.", process);
                            println!("Use 'exec -l' to list available programs.");
                        }
                    }
                }
            }
            "-l" | "--list" => {
                println!("Available programs:");
                // Here we would list the available programs.
                // For now, we just print a placeholder.
                println!(" - async_demo <char> <time to sleep>");
            }
            _ => {
                println!("Unknown option: {}", option);
                println!("Use 'help exec' to see more options.");
            }
        }
    }

    fn description(&self) -> String {
        self.manual.build_short()
    }

    fn manual(&self) -> String {
        self.manual.build_long()
    }

    fn name(&self) -> &str {
        "exec"
    }
}
