use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoSaveChoice {
    LocalOnly,
    ShareToCommunity,
    Disabled,
}

pub struct ConsentPrompt {
    selected: usize,
    options: Vec<(&'static str, AutoSaveChoice)>,
}

impl ConsentPrompt {
    pub fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                ("Save locally only", AutoSaveChoice::LocalOnly),
                ("Save locally + share to community", AutoSaveChoice::ShareToCommunity),
                ("Don't save automatically", AutoSaveChoice::Disabled),
            ],
        }
    }

    fn draw(&self) {
        // Clear screen
        print!("\x1B[2J\x1B[H");
        
        let version = env!("CARGO_PKG_VERSION");
        
        // Header
        println!("\x1B[33mWelcome to Aethr\x1B[0m");
        println!("Version {}", version);
        println!();
        
        // Description
        println!("Aethr helps you recall, fix, and organize your terminal commands.");
        println!("Enter ? for help. Aethr learns from your history.");
        println!();
        
        // Section header
        println!("\x1B[1mConfigure auto-save\x1B[0m");
        println!();
        
        // Find longest option for box width
        let max_len = self.options.iter().map(|(s, _)| s.len()).max().unwrap_or(30);
        let box_width = max_len + 6;
        
        // Top border
        println!("┌{}┐", "─".repeat(box_width));
        
        // Options inside box
        for (idx, (label, _)) in self.options.iter().enumerate() {
            let is_selected = idx == self.selected;
            let marker = if is_selected { "●" } else { "○" };
            let padding = box_width - label.len() - 4;
            
            if is_selected {
                // Highlighted - yellow/gold marker
                println!("│ \x1B[33m{}\x1B[0m {}{} │", marker, label, " ".repeat(padding));
            } else {
                // Normal - dim
                println!("│ \x1B[90m{}\x1B[0m {}{} │", marker, label, " ".repeat(padding));
            }
        }
        
        // Bottom border
        println!("└{}┘", "─".repeat(box_width));
        
        println!();
        
        // Explanation text
        println!("Aethr can automatically save your commands to improve recall and");
        println!("fix suggestions. Your data stays on your machine unless you choose");
        println!("to share with the community to help improve Aethr for everyone.");
        println!();
        
        // Question
        println!("Which option would you like?");
        println!();
        
        // Keyboard shortcuts
        println!("\x1B[90m↑↓ Navigate  Enter Select  Esc Cancel\x1B[0m");
        
        io::stdout().flush().unwrap();
    }

    fn cleanup(&self) {
        let _ = disable_raw_mode();
        print!("\x1B[?25h");
        print!("\x1B[0m");
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self) -> io::Result<AutoSaveChoice> {
        print!("\x1B[?25l"); // Hide cursor
        self.draw();
        
        if enable_raw_mode().is_err() {
            self.cleanup();
            return Ok(AutoSaveChoice::Disabled);
        }

        loop {
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                                self.cleanup();
                                return Ok(AutoSaveChoice::Disabled);
                            }
                            (KeyCode::Esc, _) => {
                                self.cleanup();
                                return Ok(AutoSaveChoice::Disabled);
                            }
                            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                                if self.selected > 0 {
                                    self.selected -= 1;
                                    self.draw();
                                }
                            }
                            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                                if self.selected < self.options.len() - 1 {
                                    self.selected += 1;
                                    self.draw();
                                }
                            }
                            (KeyCode::Enter, _) => {
                                let choice = self.options[self.selected].1;
                                self.cleanup();
                                return Ok(choice);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn show_consent() -> io::Result<AutoSaveChoice> {
    let mut prompt = ConsentPrompt::new();
    prompt.run()
}
