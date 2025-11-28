use std::io::{self, Write};
use std::time::Duration;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Professional loading spinner for CLI operations
pub struct Spinner {
    message: String,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// Create and start a new spinner with a message
    pub fn start(message: impl Into<String>) -> Self {
        let msg = message.into();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let msg_clone = msg.clone();
        
        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut idx = 0;
            
            while running_clone.load(Ordering::Relaxed) {
                let frame = frames[idx % frames.len()];
                print!("\r \x1b[36m{}\x1b[0m {}", frame, msg_clone);
                io::stdout().flush().ok();
                idx += 1;
                thread::sleep(Duration::from_millis(80));
            }
            
            // Clear the line
            let clear_len = msg_clone.len() + 10;
            print!("\r{}\r", " ".repeat(clear_len));
            io::stdout().flush().ok();
        });
        
        Self {
            message: msg,
            running,
            handle: Some(handle),
        }
    }
    
    /// Stop the spinner and show success message
    pub fn success(mut self, message: impl AsRef<str>) {
        self.stop_internal();
        println!(" \x1b[32m+\x1b[0m {}", message.as_ref());
    }
    
    /// Stop the spinner and show error message
    pub fn error(mut self, message: impl AsRef<str>) {
        self.stop_internal();
        println!(" \x1b[31mx\x1b[0m {}", message.as_ref());
    }
    
    /// Stop the spinner and show info message
    pub fn info(mut self, message: impl AsRef<str>) {
        self.stop_internal();
        println!(" \x1b[36m-\x1b[0m {}", message.as_ref());
    }
    
    /// Stop the spinner without printing anything
    pub fn stop(mut self) {
        self.stop_internal();
    }
    
    fn stop_internal(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Status messages matching enterprise CLI aesthetic
pub struct Status;

impl Status {
    pub fn success(message: impl AsRef<str>) {
        println!("  \x1b[32m+\x1b[0m {}", message.as_ref());
    }

    pub fn error(message: impl AsRef<str>) {
        println!("  \x1b[31mx\x1b[0m {}", message.as_ref());
    }

    pub fn info(message: impl AsRef<str>) {
        println!("  \x1b[36m-\x1b[0m {}", message.as_ref());
    }

    pub fn warning(message: impl AsRef<str>) {
        println!("  \x1b[33m!\x1b[0m {}", message.as_ref());
    }

    pub fn pending(message: impl AsRef<str>) {
        println!("  \x1b[36m>\x1b[0m {}", message.as_ref());
    }
}

/// Progress indicator with percentage
pub struct Progress {
    total: usize,
    current: usize,
    message: String,
}

impl Progress {
    pub fn new(total: usize, message: impl Into<String>) -> Self {
        Self {
            total,
            current: 0,
            message: message.into(),
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
        self.display();
    }

    pub fn set(&mut self, current: usize) {
        self.current = current.min(self.total);
        self.display();
    }

    fn display(&self) {
        let percent = (self.current as f32 / self.total as f32 * 100.0) as usize;
        let bar_len = 30;
        let filled = (percent * bar_len) / 100;

        print!(
            "\r\x1b[36m{:3}%\x1b[0m [",
            percent
        );

        for i in 0..bar_len {
            if i < filled {
                print!("━");
            } else {
                print!("─");
            }
        }

        println!("] {}", self.message);
        io::stdout().flush().unwrap();
    }

    pub fn finish(&self, success: bool) {
        if success {
            println!("\r\x1b[32m✓\x1b[0m 100% [{}] {}", "━".repeat(30), self.message);
        } else {
            println!("\r\x1b[31m✗\x1b[0m Failed: {}", self.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_start_stop() {
        let spinner = Spinner::start("Testing");
        std::thread::sleep(std::time::Duration::from_millis(100));
        spinner.stop();
    }

    #[test]
    fn test_progress() {
        let mut progress = Progress::new(100, "Processing");
        progress.set(50);
        assert_eq!(progress.current, 50);
    }
}
