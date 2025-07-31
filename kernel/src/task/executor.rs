//! Task executor module
use crate::{io::serial, serial_println};

use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
/// Task executor that runs tasks on a single thread
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    /// Create a new Executor
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }
    /// Spawn a new task to be executed by the executor
    pub fn spawn(&mut self, task: Task) {
        // Set the task ID
        let task_id = task.id;
        super::RUNNING_TASKS.lock().push(task_id);
        // If the task ID already exists, panic
        // because if the task ID already exists, it means
        // that there is a bug in our program
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        // Push the task ID to the task queue
        // If the task queue is full, panic
        self.task_queue.push(task_id).expect("task queue full");
    }
    /// Run the executor, executing tasks until the queue is empty
    ///
    /// This implementation is more efficient than the simple executor
    /// because it will put the CPU to sleep when there are no tasks to run
    /// or when all tasks are waiting for something to happen (e.g. I/O)
    fn run_ready_tasks(&mut self) {
        // Create a new scope to limit the lifetime of the mutable borrows
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;
        // Loop over all tasks in the task queue
        while let Some(task_id) = task_queue.pop() {
            // Get the task from the tasks map
            // If the task is not found, skip it
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            // Get the waker for the task
            // If the waker is not found, insert a new one
            // therefore, we can wake the task later without
            // having to look up the waker again
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            // Create a new context from the waker
            let mut context = Context::from_waker(waker);
            // Poll the task
            // If the task is ready, remove it from the tasks map
            // and the waker cache, else, keep it in the task queue
            // to be polled again later
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                    let mut running_tasks = super::RUNNING_TASKS.lock();
                    running_tasks.retain(|&id| id != task_id);
                }
                Poll::Pending => {}
            }
        }
    }
    /// Run the executor
    /// This will run all tasks until they are all completed
    /// and then put the CPU to sleep until there are new tasks
    /// to run
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }
    /// Put the CPU to sleep until there are new tasks to run
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};
        // Disable interrupts to prevent race conditions
        interrupts::disable();
        // Check if the task queue is empty
        if self.task_queue.is_empty() {
            // If it is, enable interrupts and put the CPU to sleep
            enable_and_hlt();
        } else {
            // If there are tasks to run, only enable interrupts
            interrupts::enable();
        }
    }
}

pub fn run_executor() -> ! {
    loop {
        // Get a mutable reference to the global executor
        let mut executor = super::GLOBAL_EXECUTOR.lock();
        // Run the executor
        executor.run_ready_tasks();
        executor.sleep_if_idle();

        let mut task_remove_queue = super::REM_TASK_Q.lock();
        while let Some(task_id) = task_remove_queue.pop() {
            let task_id = TaskId::from(task_id);
            if executor.tasks.remove(&task_id).is_none() {
                serial_println!("WARNING: task with ID {:?} not found in executor", task_id);
            }
            executor.waker_cache.remove(&task_id);
            let mut running_tasks = super::RUNNING_TASKS.lock();
            running_tasks.retain(|&id| id != task_id);
            serial_println!("Removed task with ID {:?}", task_id);
        }

        let mut task_add_queue = super::ADD_TASK_Q.lock();
        while let Some(task) = task_add_queue.pop() {
            executor.spawn(task);
        }
    }
}

unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

/// Task waker that can wake up tasks
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    /// Wake up the task associated with this waker
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
    /// Create a new TaskWaker
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
