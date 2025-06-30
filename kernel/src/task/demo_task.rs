//! A simple example task that demonstrates cooperative multitasking.

use crate::interrupts::handlers::LAPIC_TIMER_SLEEP_COUNTER;
use crate::task::TaskId;
use crate::{print, serial_println};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;
use core::task::{Context, Poll};
use spin::Mutex;

/// A simple future that yields control back to the executor a number of times.
/// This acts as a basic, non-blocking sleep, allowing other tasks to run.

pub struct Sleep {
    pub remaining: u64,
    pub task_id: TaskId, // some unique identifier
    pub registered: bool,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use crate::interrupts::handlers::{SleepEntry, SLEEP_TASKS};
        if self.remaining == 0 {
            return Poll::Ready(());
        }
        // If not registered yet, store ourselves in SLEEP_TASKS with the waker
        if !self.registered {
            let mut tasks = SLEEP_TASKS.lock();
            tasks.insert(
                self.task_id.as_u64(),
                SleepEntry {
                    remaining: self.remaining,
                    waker: cx.waker().clone(),
                },
            );
            self.registered = true;
        }
        Poll::Pending
    }
}

/// Create a new Sleep future for the given duration, associated with a task ID.
pub fn sleep(duration: u64, task_id: TaskId) -> Sleep {
    todo!("FINISH ME");
    crate::timer::lapic::LAPICTimer::start_periodic_timer();
    Sleep {
        remaining: duration,
        task_id,
        registered: false,
    }
}

// Add a way to signal tasks to terminate
lazy_static::lazy_static! {
    static ref CANCEL_SIGNALS: Mutex<alloc::collections::BTreeMap<u64, AtomicBool>> =
        Mutex::new(alloc::collections::BTreeMap::new());
}

/// A simple demo task that prints a character to the screen, sleeps for a bit,
/// and then repeats. This demonstrates that the executor is running multiple
/// tasks by interleaving their execution.
pub async fn demo_task_runner(character_to_print: char, sleep_duration: u64, id: TaskId) {
    use crate::task::TaskId;
    use core::task::Waker;

    // Get current task ID for cancellation support
    let task_id = id;

    // Register this task for cancellation support
    {
        let mut signals = CANCEL_SIGNALS.lock();
        signals.insert(task_id.0, AtomicBool::new(false));
        serial_println!("Task {} started", task_id.0);
    }

    loop {
        // Check if we should terminate
        serial_println!("Task {} checking for cancellation", task_id.0);
        let should_cancel = {
            let signals = CANCEL_SIGNALS.lock();
            if let Some(signal) = signals.get(&task_id.0) {
                signal.load(Ordering::Relaxed)
            } else {
                false
            }
        };
        serial_println!(
            "Task {} checking for cancellation: {}",
            task_id.0,
            should_cancel
        );

        if should_cancel {
            serial_println!("Task {} terminating", task_id.0);
            break;
        }

        print!("{}", character_to_print);
        sleep(sleep_duration, id).await;
        serial_println!("Task {} woke up", task_id.0);
    }

    // Clean up
    let mut signals = CANCEL_SIGNALS.lock();
    signals.remove(&task_id.0);
}

/// Cancel a specific task by ID
pub fn cancel_task(task_id: u64) -> bool {
    let signals = CANCEL_SIGNALS.lock();
    if let Some(signal) = signals.get(&task_id) {
        signal.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}
