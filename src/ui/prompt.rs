use crossterm::{
    cursor::{Hide, Show, MoveTo, MoveToColumn, SavePosition, RestorePosition},
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    style::Print,
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write, stdout};

use crate::utils::config::{self, AethrConfig};

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
    username: Option<String>,
    header_lines: u16,
}

impl InteractivePrompt {
    pub fn new() -> Self {
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .ok();
        
        Self {
            input: String::new(),
            commands: vec![
                SlashCommand { name: "recall", description: "Search command history" },
                SlashCommand { name: "fix", description: "Fix a terminal error" },
                SlashCommand { name: "import", description: "Import shell history" },
                SlashCommand { name: "init", description: "Initialize Aethr" },
                SlashCommand { name: "status", description: "Check Aethr status" },
                SlashCommand { name: "clear", description: "Clear conversation" },
                SlashCommand { name: "help", description: "Show help" },
                SlashCommand { name: "exit", description: "Exit Aethr" },
            ],
            selected_command: 0,
            show_menu: false,
            username,
            header_lines: 0,
        }
    }

    fn get_term_width(&self) -> u16 {
        size().map(|(w, _)| w).unwrap_or(80)
    }

    fn print_header(&mut self) {
        let version = env!("CARGO_PKG_VERSION");
        let term_width = self.get_term_width() as usize;
        
        // Yellow header
        println!("\x1B[33mWelcome to Aethr CLI\x1B[0m");
        println!("Version {}", version);
        println!();
        
        // Description
        println!("Aethr helps you recall, fix, and organize your terminal commands. Describe");
        println!("what you need or enter ? for help. Aethr learns from your history.");
        println!();
        
        // Status bullets
        let token_path = config::get_token_path();
        let logged_in = token_path.exists() && 
            std::fs::read_to_string(&token_path)
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        
        if logged_in {
            if let Some(ref user) = self.username {
                println!("\x1B[33m●\x1B[0m Logged in as user: \x1B[1m{}\x1B[0m", user);
            } else {
                println!("\x1B[33m●\x1B[0m Logged in");
            }
        } else {
            println!("\x1B[90m○\x1B[0m Not logged in \x1B[90m(run aethr login)\x1B[0m");
        }
        
        let config = AethrConfig::load();
        if config.auto_save {
            println!("\x1B[33m●\x1B[0m Auto-save enabled");
        }
        
        println!();
        
        // Current directory + model badge
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~".to_string());
        
        let model_text = "claude-sonnet-4.5 (1x)";
        let max_path = term_width.saturating_sub(model_text.len() + 4);
        let display_path = if cwd.len() > max_path {
            format!("...{}", &cwd[cwd.len().saturating_sub(max_path - 3)..])
        } else {
            cwd
        };
        
        let spaces = term_width.saturating_sub(display_path.len() + model_text.len());
        println!("{}{}\x1B[90m{}\x1B[0m", display_path, " ".repeat(spaces), model_text);
        
        // Thick separator line
        println!("{}", "━".repeat(term_width));
        
        self.header_lines = 12; // Approximate lines used by header
    }

    fn draw_prompt_area(&self) {
        let term_width = self.get_term_width() as usize;
        
        // Input line with cursor block
        print!("> ");
        if self.input.is_empty() {
            print!("\x1B[7m \x1B[0m"); // Inverted space as cursor
            print!("\x1B[90mEnter / for commands\x1B[0m");
        } else {
            print!("{}", self.input);
            print!("\x1B[7m \x1B[0m"); // Cursor at end
        }
        println!();
        
        // Command dropdown menu
        if self.show_menu {
            let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
            let filtered: Vec<_> = self.commands.iter()
                .filter(|c| filter.is_empty() || c.name.to_lowercase().starts_with(&filter.to_lowercase()))
                .collect();
            
            if !filtered.is_empty() {
                for (i, cmd) in filtered.iter().enumerate() {
                    let is_selected = i == self.selected_command;
                    
                    if is_selected {
                        // Yellow square indicator for selected
                        print!("\x1B[33m█\x1B[0m ");
                        print!("\x1B[36m/{:<18}\x1B[0m", cmd.name);
                        println!("\x1B[33m{}\x1B[0m", cmd.description);
                    } else {
                        print!("\x1B[90m█\x1B[0m ");
                        print!("\x1B[36m/{:<18}\x1B[0m", cmd.name);
                        println!("\x1B[90m{}\x1B[0m", cmd.description);
                    }
                }
            }
        }
        
        // Bottom separator
        println!("{}", "━".repeat(term_width));
        
        // Shortcuts footer
        println!("\x1B[1mCtrl+C\x1B[0m Exit  \x1B[90m·\x1B[0m  \x1B[1mCtrl+R\x1B[0m Expand recent");
        
        io::stdout().flush().unwrap();
    }

    fn redraw(&self) {
        // Move cursor up to overwrite prompt area, clear, redraw
        let lines_to_clear = if self.show_menu {
            3 + self.get_filtered_commands(&self.get_filter()).len() as u16
        } else {
            3
        };
        
        // Move up and clear
        print!("\x1B[{}A", lines_to_clear + 3);
        print!("\x1B[J"); // Clear from cursor to end
        
        self.draw_prompt_area();
    }

    fn get_filter(&self) -> String {
        if self.input.starts_with('/') {
            self.input[1..].to_string()
        } else {
            String::new()
        }
    }

    fn get_filtered_commands(&self, filter: &str) -> Vec<&SlashCommand> {
        self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.to_lowercase().starts_with(&filter.to_lowercase()))
            .collect()
    }

    fn cleanup(&self) {
        let _ = disable_raw_mode();
        print!("\x1B[?25h"); // Show cursor
        print!("\x1B[0m");
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self) -> io::Result<Option<(String, String)>> {
        // Print header (doesn't clear screen)
        self.print_header();
        self.draw_prompt_area();
        
        // Hide cursor and enable raw mode
        print!("\x1B[?25l");
        io::stdout().flush()?;
        
        if enable_raw_mode().is_err() {
            self.cleanup();
            return Ok(None);
        }

        let result = self.event_loop();
        self.cleanup();
        println!(); // New line after exit
        result
    }

    fn event_loop(&mut self) -> io::Result<Option<(String, String)>> {
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Only handle key press events (not release)
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    
                    match (key.code, key.modifiers) {
                        // Ctrl+C - Exit
                        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                            return Ok(None);
                        }
                        // Escape - Close menu or exit
                        (KeyCode::Esc, _) => {
                            if self.show_menu {
                                self.show_menu = false;
                                self.input.clear();
                                self.selected_command = 0;
                                self.redraw();
                            } else {
                                return Ok(None);
                            }
                        }
                        // Arrow Up
                        (KeyCode::Up, _) => {
                            if self.show_menu && self.selected_command > 0 {
                                self.selected_command -= 1;
                                self.redraw();
                            }
                        }
                        // Arrow Down  
                        (KeyCode::Down, _) => {
                            if self.show_menu {
                                let filter = self.get_filter();
                                let max = self.get_filtered_commands(&filter).len().saturating_sub(1);
                                if self.selected_command < max {
                                    self.selected_command += 1;
                                    self.redraw();
                                }
                            }
                        }
                        // Enter - Select or submit
                        (KeyCode::Enter, _) => {
                            if self.show_menu {
                                let filter = self.get_filter();
                                let filtered = self.get_filtered_commands(&filter);
                                if let Some(cmd) = filtered.get(self.selected_command) {
                                    if cmd.name == "exit" {
                                        return Ok(None);
                                    }
                                    if cmd.name == "clear" {
                                        // Clear and redraw everything
                                        print!("\x1B[2J\x1B[H");
                                        self.input.clear();
                                        self.show_menu = false;
                                        self.selected_command = 0;
                                        self.print_header();
                                        self.draw_prompt_area();
                                        continue;
                                    }
                                    return Ok(Some((cmd.name.to_string(), String::new())));
                                }
                            } else if !self.input.is_empty() {
                                let input = self.input.clone();
                                if input.starts_with('/') {
                                    let cmd = &input[1..];
                                    return Ok(Some((cmd.to_string(), String::new())));
                                } else {
                                    return Ok(Some(("query".to_string(), input)));
                                }
                            }
                        }
                        // Tab - Autocomplete
                        (KeyCode::Tab, _) => {
                            if self.show_menu {
                                let filter = self.get_filter();
                                let filtered = self.get_filtered_commands(&filter);
                                if let Some(cmd) = filtered.get(self.selected_command) {
                                    self.input = format!("/{}", cmd.name);
                                    self.show_menu = false;
                                    self.redraw();
                                }
                            }
                        }
                        // Backspace
                        (KeyCode::Backspace, _) => {
                            if !self.input.is_empty() {
                                self.input.pop();
                                self.show_menu = self.input.starts_with('/');
                                if !self.show_menu {
                                    self.selected_command = 0;
                                }
                                self.redraw();
                            }
                        }
                        // Regular character
                        (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                            self.input.push(c);
                            
                            if self.input.starts_with('/') {
                                self.show_menu = true;
                                self.selected_command = 0;
                            } else {
                                self.show_menu = false;
                            }
                            
                            self.redraw();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn run_interactive() -> io::Result<Option<(String, String)>> {
    let mut prompt = InteractivePrompt::new();
    prompt.run()
}
