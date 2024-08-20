// TO DO : https://os.phil-opp.com/async-await/#possible-extensions
//! Task module

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;
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
    /// Poll the task
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// An identifier for a task
struct TaskId(u64);

impl TaskId {
    /// Create a new TaskId with a unique identifier
    fn new() -> Self {
        // This will create a new unique identifier for each task
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
