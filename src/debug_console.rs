//! Reads debug commands from stdin without blocking the game loop.
//!
//! A background thread does the blocking line-read and ships each line over a
//! channel; the game loop polls with `poll()` every frame. On wasm there is no
//! stdin, so the thread is skipped and `poll()` always returns `None`.

use std::sync::mpsc::{self, Receiver, TryRecvError};

pub struct DebugConsole {
    rx: Option<Receiver<String>>,
}

impl DebugConsole {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::io::BufRead;
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let stdin = std::io::stdin();
                for line in stdin.lock().lines() {
                    // stop the thread on read error or once the receiver is dropped
                    match line {
                        Ok(line) => {
                            if tx.send(line).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
            Self { rx: Some(rx) }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = mpsc::channel::<String>;
            Self { rx: None }
        }
    }

    /// Returns the next command line entered since the last call, or `None`.
    /// Never blocks.
    pub fn poll(&self) -> Option<String> {
        let rx = self.rx.as_ref()?;
        match rx.try_recv() {
            Ok(line) => Some(line),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }
}
