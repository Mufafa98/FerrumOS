//! Simple Executor module

use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
/// A simple executor that runs tasks on a single thread
pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}
impl SimpleExecutor {
    /// Create a new SimpleExecutor with an empty task queue
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }
    /// Spawn a new task to be executed by the executor
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
    /// Run the executor, executing tasks until the queue is empty
    ///
    /// This is highly inefficient keeping the CPU busy. Can be improved
    /// see executor.rs for a better implementation
    pub fn run(&mut self) {
        // Keep running tasks until the task queue is empty
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

/// Create a dummy RawWaker
fn dummy_raw_waker() -> RawWaker {
    /// A dummy RawWakerVTable that does nothing
    fn no_op(_: *const ()) {}
    /// Clone the dummy RawWaker
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    // Create a RawWakerVTable with the clone and no_op functions
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    // Create a RawWaker with a null pointer and the vtable
    RawWaker::new(0 as *const (), vtable)
}
/// Create a dummy Waker
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
