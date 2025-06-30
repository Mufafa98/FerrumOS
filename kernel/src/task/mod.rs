// TO DO : https://os.phil-opp.com/async-await/#possible-extensions
//! Task module

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod demo_task;
pub mod executor;
pub mod keyboard;
pub mod simple_executor;

use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
    /// A static instance of the ScancodeStream
    pub static ref GLOBAL_EXECUTOR: Mutex<executor::Executor> = Mutex::new(executor::Executor::new());
    pub static ref ADD_TASK_Q: Mutex<crossbeam_queue::ArrayQueue<Task>> =
        Mutex::new(crossbeam_queue::ArrayQueue::new(100));
    pub static ref REM_TASK_Q: Mutex<crossbeam_queue::ArrayQueue<u64>> =
        Mutex::new(crossbeam_queue::ArrayQueue::new(100));
    pub static ref RUNNING_TASKS: Mutex<Vec<TaskId>> =
        Mutex::new(Vec::new());
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

/// A task that can be executed by the executor
pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Create a new Task
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }
    pub fn new_with_id(id: u64, future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::from(id),
            future: Box::pin(future),
        }
    }
    /// Poll the task
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
    pub fn id(&self) -> u64 {
        self.id.as_u64()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// An identifier for a task
pub struct TaskId(u64);

impl TaskId {
    /// Create a new TaskId with a unique identifier
    pub fn new() -> Self {
        // This will create a new unique identifier for each task
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    fn from(id: u64) -> Self {
        TaskId(id)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}
