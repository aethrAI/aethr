use std::io::{self, Write};
use std::time::Duration;
use std::thread;

/// Professional loading spinner for CLI operations
/// Inspired by Claude, Gemini, and other enterprise CLIs
pub struct Spinner {
    message: String,
    spinner_type: SpinnerType,
}

#[derive(Clone, Copy)]
pub enum SpinnerType {
    /// Animated dots: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
    Braille,
    /// Smooth rotating line: ⠋ ⠙ ⠚ ⠒ ⠂ ⠂ ⠒ ⠲ ⠴ ⠦ ⠖ ⠒ ⠐ ⠐ ⠒ ⠓ ⠋
    Dots,
    /// Arc animation: ◜ ◠ ◝ ◞ ◡ ◟
    Arc,
    /// Classic: | / - \
    Line,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            spinner_type: SpinnerType::Braille,
        }
    }

    pub fn with_type(mut self, spinner_type: SpinnerType) -> Self {
        self.spinner_type = spinner_type;
        self
    }

    /// Get the spinner frames based on type
    fn frames(&self) -> &'static [&'static str] {
        match self.spinner_type {
            SpinnerType::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerType::Dots => &["⢿", "⣟", "⣯", "⣷", "⣾", "⣽", "⣻"],
            SpinnerType::Arc => &["◜ ", "◠ ", "◝ ", "◞ ", "◡ ", "◟ "],
            SpinnerType::Line => &["|", "/", "-", "\\"],
        }
    }

    /// Run spinner for a duration or until manually stopped
    pub fn start(&self) {
        let frames = self.frames();
        let mut idx = 0;

        loop {
            let frame = frames[idx % frames.len()];
            print!("\r\x1b[36m{}\x1b[0m {}", frame, self.message);
            io::stdout().flush().unwrap();

            thread::sleep(Duration::from_millis(80));
            idx += 1;

            // Check for timeout or break signal
            // In real usage, this would be tied to a completion event
        }
    }

    /// Run spinner in the background (returns immediately)
    /// Note: Real implementation would use a background thread with Arc<Mutex>
    pub fn start_background(&self) {
        // This is a placeholder - in production, use crossbeam or tokio
        // For now, just show message
        println!("\r\x1b[36m⠋\x1b[0m {}", self.message);
    }
}

/// Status messages matching enterprise CLI aesthetic
pub struct Status;

impl Status {
    pub fn success(message: impl AsRef<str>) {
        println!("\r\x1b[32m✓\x1b[0m {}", message.as_ref());
    }

    pub fn error(message: impl AsRef<str>) {
        println!("\r\x1b[31m✗\x1b[0m {}", message.as_ref());
    }

    pub fn info(message: impl AsRef<str>) {
        println!("\r\x1b[36mℹ\x1b[0m {}", message.as_ref());
    }

    pub fn warning(message: impl AsRef<str>) {
        println!("\r\x1b[33m⚠\x1b[0m {}", message.as_ref());
    }

    pub fn pending(message: impl AsRef<str>) {
        println!("\r\x1b[36m→\x1b[0m {}", message.as_ref());
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
    fn test_spinner_frames() {
        let spinner = Spinner::new("Testing");
        assert!(!spinner.frames().is_empty());
    }

    #[test]
    fn test_progress() {
        let mut progress = Progress::new(100, "Processing");
        progress.set(50);
        assert_eq!(progress.current, 50);
    }
}
