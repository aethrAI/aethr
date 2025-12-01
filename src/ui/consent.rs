use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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
    options: Vec<(&'static str, &'static str, AutoSaveChoice)>,
}

impl ConsentPrompt {
    pub fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                ("Auto-save locally", "Commands saved on your machine only", AutoSaveChoice::LocalOnly),
                ("Auto-save + Community", "Help improve Aethr for everyone", AutoSaveChoice::ShareToCommunity),
                ("Disable auto-save", "Import history manually later", AutoSaveChoice::Disabled),
            ],
        }
    }

    fn print_logo(&self) {
        println!();
        println!("\x1B[1;37m        █████╗ ███████╗████████╗██╗  ██╗██████╗ \x1B[0m");
        println!("\x1B[1;37m       ██╔══██╗██╔════╝╚══██╔══╝██║  ██║██╔══██╗\x1B[0m");
        println!("\x1B[1;37m       ███████║█████╗     ██║   ███████║██████╔╝\x1B[0m");
        println!("\x1B[1;37m       ██╔══██║██╔══╝     ██║   ██╔══██║██╔══██╗\x1B[0m");
        println!("\x1B[1;37m       ██║  ██║███████╗   ██║   ██║  ██║██║  ██║\x1B[0m");
        println!("\x1B[1;37m       ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝\x1B[0m");
        println!();
        println!("\x1B[2m       Terminal Intelligence That Learns\x1B[0m");
        println!();
    }

    fn draw(&self) {
        // Clear screen
        print!("\x1B[2J\x1B[H");
        
        self.print_logo();
        
        println!("  \x1B[1mSetup\x1B[0m");
        println!();
        println!("  Aethr can automatically save your commands to improve");
        println!("  recall and fix suggestions.");
        println!();
        println!("  Your data stays on your machine unless you choose to share.");
        println!();
        println!("  \x1B[90mSelect an option:\x1B[0m");
        println!();
        
        for (idx, (label, desc, _)) in self.options.iter().enumerate() {
            if idx == self.selected {
                println!("  \x1B[46;30m ● {:<28} \x1B[0m", label);
                println!("    \x1B[36m{}\x1B[0m", desc);
            } else {
                println!("  \x1B[90m ○\x1B[0m {}", label);
                println!("    \x1B[90m{}\x1B[0m", desc);
            }
            println!();
        }
        
        println!();
        println!("  \x1B[90m↑↓ Navigate  Enter Select\x1B[0m");
        
        io::stdout().flush().unwrap();
    }

    fn cleanup(&self) {
        let _ = disable_raw_mode();
        print!("\x1B[?25h");
        print!("\x1B[0m");
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self) -> io::Result<AutoSaveChoice> {
        self.draw();
        
        if enable_raw_mode().is_err() {
            self.cleanup();
            return Ok(AutoSaveChoice::Disabled);
        }

        loop {
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
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
                                let choice = self.options[self.selected].2;
                                self.cleanup();
                                return Ok(choice);
                            }
                            (KeyCode::Char('1'), _) => {
                                self.selected = 0;
                                self.draw();
                            }
                            (KeyCode::Char('2'), _) => {
                                self.selected = 1;
                                self.draw();
                            }
                            (KeyCode::Char('3'), _) => {
                                self.selected = 2;
                                self.draw();
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
