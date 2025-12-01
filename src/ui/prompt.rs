use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};

use crate::utils::config;

#[derive(Clone)]
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct InteractivePrompt {
    input: String,
    commands: Vec<SlashCommand>,
    selected_command: usize,
    show_menu: bool,
}

impl InteractivePrompt {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            commands: vec![
                SlashCommand { name: "recall", description: "Search command history" },
                SlashCommand { name: "fix", description: "Fix a terminal error" },
                SlashCommand { name: "import", description: "Import shell history" },
                SlashCommand { name: "init", description: "Initialize Aethr" },
                SlashCommand { name: "status", description: "Check Aethr status" },
                SlashCommand { name: "help", description: "Show help" },
                SlashCommand { name: "exit", description: "Exit Aethr" },
            ],
            selected_command: 0,
            show_menu: false,
        }
    }

    fn print_header(&self) {
        let version = env!("CARGO_PKG_VERSION");
        
        // Clear screen
        print!("\x1B[2J\x1B[H");
        
        println!();
        println!("  \x1B[1;36m┌─────────────────────────────────────────┐\x1B[0m");
        println!("  \x1B[1;36m│\x1B[0m  \x1B[1mAethr\x1B[0m v{}                          \x1B[1;36m│\x1B[0m", version);
        println!("  \x1B[1;36m│\x1B[0m  Terminal Intelligence                 \x1B[1;36m│\x1B[0m");
        println!("  \x1B[1;36m└─────────────────────────────────────────┘\x1B[0m");
        println!();
        
        let token_path = config::get_token_path();
        let logged_in = token_path.exists() && 
            std::fs::read_to_string(&token_path)
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        
        if logged_in {
            println!("  \x1B[32m●\x1B[0m Logged in");
        } else {
            println!("  \x1B[33m○\x1B[0m Not logged in");
        }
        println!();
    }

    fn draw(&self) {
        // Clear from line 8 to bottom
        print!("\x1B[8;1H\x1B[J");
        
        // Input prompt  
        print!("  \x1B[36m❯\x1B[0m ");
        if self.input.is_empty() {
            print!("\x1B[90mType / for commands...\x1B[0m");
        } else {
            print!("{}", self.input);
        }
        println!();
        println!();
        
        // Command menu
        if self.show_menu {
            let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
            let filtered: Vec<_> = self.commands.iter()
                .filter(|c| filter.is_empty() || c.name.starts_with(filter))
                .collect();
            
            for (i, cmd) in filtered.iter().enumerate() {
                if i == self.selected_command {
                    println!("  \x1B[46;30m /{:<10} {:<25} \x1B[0m", cmd.name, cmd.description);
                } else {
                    println!("   \x1B[36m/{}\x1B[0m \x1B[90m{}\x1B[0m", cmd.name, cmd.description);
                }
            }
            println!();
        }
        
        // Footer
        println!("  \x1B[90m↑↓ Navigate  Enter Select  Ctrl+C Exit\x1B[0m");
        
        io::stdout().flush().unwrap();
    }

    fn get_filtered_commands(&self, filter: &str) -> Vec<&SlashCommand> {
        self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.starts_with(filter))
            .collect()
    }

    fn cleanup(&self) {
        let _ = disable_raw_mode();
        print!("\x1B[?25h"); // Show cursor
        print!("\x1B[0m");   // Reset colors
        println!();
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self) -> io::Result<Option<(String, String)>> {
        self.print_header();
        self.draw();
        
        if enable_raw_mode().is_err() {
            self.cleanup();
            return Ok(None);
        }

        let result = self.event_loop();
        self.cleanup();
        result
    }

    fn event_loop(&mut self) -> io::Result<Option<(String, String)>> {
        loop {
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                                return Ok(None);
                            }
                            (KeyCode::Esc, _) => {
                                if self.show_menu {
                                    self.show_menu = false;
                                    self.input.clear();
                                    self.selected_command = 0;
                                    self.draw();
                                } else {
                                    return Ok(None);
                                }
                            }
                            (KeyCode::Up, _) => {
                                if self.show_menu && self.selected_command > 0 {
                                    self.selected_command -= 1;
                                    self.draw();
                                }
                            }
                            (KeyCode::Down, _) => {
                                if self.show_menu {
                                    let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                    let max = self.get_filtered_commands(filter).len().saturating_sub(1);
                                    if self.selected_command < max {
                                        self.selected_command += 1;
                                        self.draw();
                                    }
                                }
                            }
                            (KeyCode::Enter, _) => {
                                if self.show_menu {
                                    let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                    let filtered = self.get_filtered_commands(filter);
                                    if let Some(cmd) = filtered.get(self.selected_command) {
                                        if cmd.name == "exit" {
                                            return Ok(None);
                                        }
                                        return Ok(Some((cmd.name.to_string(), String::new())));
                                    }
                                } else if !self.input.is_empty() {
                                    let input = self.input.clone();
                                    if input.starts_with('/') {
                                        let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
                                        return Ok(Some((parts[0].to_string(), parts.get(1).unwrap_or(&"").to_string())));
                                    } else {
                                        return Ok(Some(("query".to_string(), input)));
                                    }
                                }
                            }
                            (KeyCode::Tab, _) => {
                                if self.show_menu {
                                    let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                    let filtered = self.get_filtered_commands(filter);
                                    if let Some(cmd) = filtered.get(self.selected_command) {
                                        self.input = format!("/{}", cmd.name);
                                        self.show_menu = false;
                                        self.draw();
                                    }
                                }
                            }
                            (KeyCode::Backspace, _) => {
                                if !self.input.is_empty() {
                                    self.input.pop();
                                    self.show_menu = self.input.starts_with('/');
                                    self.selected_command = 0;
                                    self.draw();
                                }
                            }
                            (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                                self.input.push(c);
                                if self.input == "/" || self.input.starts_with('/') {
                                    self.show_menu = true;
                                    self.selected_command = 0;
                                } else {
                                    self.show_menu = false;
                                }
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

pub fn run_interactive() -> io::Result<Option<(String, String)>> {
    let mut prompt = InteractivePrompt::new();
    prompt.run()
}
