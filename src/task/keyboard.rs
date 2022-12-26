use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use crate::{print, shell, warning, WRITER};
use core::pin::Pin;
use core::task::{Poll, Context};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use crate::reading::READER;
use crate::shell::SHELL_HISTORY;

static WAKER: AtomicWaker = AtomicWaker::new();
pub static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            warning("WARNING: Scancode queue full. Dropping keyboard input.");
        } else {
            WAKER.wake();
        }
    } else {
        warning("WARNING: Scancode queue uninitialized.");
    }
}

pub async fn input_handler() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1,
                                     HandleControl::Ignore);
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                if scancode == 0x4B {
                    // Handle arrow left
                    x86_64::instructions::interrupts::without_interrupts(|| {
                        WRITER.lock().move_left();
                    });
                } else if scancode == 0x4D {
                    // Handle arrow right
                    x86_64::instructions::interrupts::without_interrupts(|| {
                        WRITER.lock().move_right();
                    });
                } else if scancode == 0x48 {
                    // Handle arrow up
                    let history = shell::SHELL_HISTORY.lock().history.clone();

                    if history.len() > 0 {
                        let mut index = shell::SHELL_HISTORY.lock().index;

                        if index == 0 {
                            return;
                        }

                        index -= 1;

                        shell::SHELL_HISTORY.lock().index = index;

                        let history = history[index as usize].clone();

                        x86_64::instructions::interrupts::without_interrupts(|| {
                            let row = WRITER.lock().row_position;
                            WRITER.lock().clear_row(row);
                            WRITER.lock().column_position = 0;
                            print!("> {}", history);
                            READER.lock().column_position_start = 2;
                        });

                        shell::SHELL_HISTORY.lock().index = index;
                    }
                } else if scancode == 0x50 {
                    // Handle arrow down
                    let history = shell::SHELL_HISTORY.lock().history.clone();

                    if history.len() > 0 {
                        let mut index = shell::SHELL_HISTORY.lock().index;

                        if history.len() as u64 - 1 == index {
                            return;
                        }

                        index += 1;
                        shell::SHELL_HISTORY.lock().index = index;

                        let history = history[index as usize].clone();

                        x86_64::instructions::interrupts::without_interrupts(|| {
                            let row = WRITER.lock().row_position;
                            WRITER.lock().clear_row(row);
                            WRITER.lock().column_position = 0;
                            print!("> {}", history);
                            READER.lock().column_position_start = 2;
                        });

                        shell::SHELL_HISTORY.lock().index = index;
                    }
                } else if scancode == 0x0E && READER.lock().awaits_input {
                    // Handle backspace
                    x86_64::instructions::interrupts::without_interrupts(|| {
                        WRITER.lock().backspace();
                    });
                } else {
                    if READER.lock().awaits_input {
                        match key {
                            DecodedKey::Unicode(character) => print!("{}", character),
                            DecodedKey::RawKey(key) => print!("{:?}", key)
                        }

                        if scancode == 0x1C {
                            // Handle Enter
                            READER.lock().awaits_input = false;
                            shell::run();
                        }
                    }
                }
            }
        }
    }

    print!("!");
}

pub struct ScancodeStream {
    _private: ()
}

impl ScancodeStream {
    pub fn new() -> Self {
        ScancodeStream {
            _private: ()
        }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("Not initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            },
            Err(crossbeam_queue::PopError) => Poll::Pending
        }
    }
}