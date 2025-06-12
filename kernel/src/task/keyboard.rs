//! Keyboard task module
use crate::{print, println};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;

/// The scancode queue for keyboard input
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
/// Add a scancode to the scancode queue
pub(crate) fn add_scancode(scancode: u8) {
    // Try to get the scancode queue
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            // If the scancode queue is full, print a warning
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // Else, wake the keyboard task
            WAKER.wake();
        }
    } else {
        // If the scancode queue is not initialized, print a warning
        println!("WARNING: scancode queue uninitialized");
    }
}
/// The scancode stream
pub struct ScancodeStream {
    _private: (),
}
impl ScancodeStream {
    /// Create a new ScancodeStream
    pub fn new() -> Self {
        // Initialize the scancode queue
        // This should only be called once
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    // The scancode stream will yield u8 values
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<u8>> {
        // Try to get the scancode queue
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");
        // Try to pop a scancode from the queue
        if let Some(scancode) = queue.pop() {
            // If a scancode was popped, return it
            return Poll::Ready(Some(scancode));
        }
        // If no scancode was popped, register the waker
        // and try to pop the scancode again. If the scancode
        // is still not available, return pending else
        // return ready with the scancode and clear the waker
        WAKER.register(&context.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}
/// The waker for the keyboard task
static WAKER: AtomicWaker = AtomicWaker::new();

lazy_static!(
    /// Last command
    pub static ref LAST_COMMAND: Mutex<String> = Mutex::new(String::new());
);

/// Print keypresses
pub async fn print_keypresses() {
    // Create a new scancode stream
    let mut scancodes = ScancodeStream::new();
    // Create a new keyboard with the US layout
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    // Loop over all scancodes
    while let Some(scancode) = scancodes.next().await {
        // Try to add the scancode to the keyboard
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            // If the scancode was added, try to get the key
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // If the key was found, print it
                match key {
                    DecodedKey::Unicode(character) => {
                        if character == '\n' {
                            print!("\n");
                            crate::shell::execute_command(&LAST_COMMAND.lock());
                            LAST_COMMAND.lock().clear();
                        } else if character == '\x08' {
                            // Backspace
                            let mut last_command = LAST_COMMAND.lock();
                            if !last_command.is_empty() {
                                last_command.pop();
                                print!("\x08 \x08");
                            }
                        } else {
                            LAST_COMMAND.lock().push(character);
                            print!("{}", character);
                        }
                    }
                    DecodedKey::RawKey(key) => {
                        // print!("{:?}", key);
                    }
                }
            }
        }
    }
}
