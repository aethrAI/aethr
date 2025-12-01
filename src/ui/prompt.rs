use crossterm::{
    cursor::{Hide, Show, MoveTo},
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size},
    ExecutableCommand,
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
}

impl InteractivePrompt {
    pub fn new() -> Self {
        // Try to get username from git config or env
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
                SlashCommand { name: "clear", description: "Clear the screen" },
                SlashCommand { name: "help", description: "Show help" },
                SlashCommand { name: "exit", description: "Exit Aethr" },
            ],
            selected_command: 0,
            show_menu: false,
            username,
        }
    }

    fn draw_full(&self) {
        let mut out = stdout();
        let _ = out.execute(Clear(ClearType::All));
        let _ = out.execute(MoveTo(0, 0));
        
        let version = env!("CARGO_PKG_VERSION");
        
        // Header - Yellow "Welcome to Aethr"
        println!("\x1B[33mWelcome to Aethr CLI\x1B[0m");
        println!("Version {} ", version);
        println!();
        
        // Description
        println!("Aethr helps you recall, fix, and organize your terminal commands. Describe");
        println!("what you need or enter ? for help. Aethr learns from your history.");
        println!();
        
        // Status section
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
        
        // Current directory and model badge
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~".to_string());
        
        // Truncate path if too long
        let (term_width, _) = size().unwrap_or((80, 24));
        let model_badge = "\x1B[90mclaude-sonnet-4.5 (1x)\x1B[0m";
        let model_len = 18; // Length without ANSI codes
        let max_path_len = (term_width as usize).saturating_sub(model_len + 4);
        
        let display_path = if cwd.len() > max_path_len {
            format!("...{}", &cwd[cwd.len().saturating_sub(max_path_len - 3)..])
        } else {
            cwd
        };
        
        // Print path on left, model badge on right
        let padding = (term_width as usize).saturating_sub(display_path.len() + model_len + 2);
        println!("{}{:>pad$}", display_path, model_badge, pad = padding + model_len + 6);
        
        // Separator line
        println!("\x1B[90m{}\x1B[0m", "─".repeat(term_width as usize));
        
        self.draw_input_area();
    }

    fn draw_input_area(&self) {
        let (term_width, _) = size().unwrap_or((80, 24));
        
        // Prompt input
        print!("> ");
        if self.input.is_empty() {
            print!("\x1B[90m█\x1B[0mEnter @ to mention files or / for commands");
        } else {
            print!("{}\x1B[90m█\x1B[0m", self.input);
        }
        println!();
        
        // Command menu dropdown (two-column style)
        if self.show_menu {
            let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
            let filtered: Vec<_> = self.commands.iter()
                .filter(|c| filter.is_empty() || c.name.to_lowercase().contains(&filter.to_lowercase()))
                .collect();
            
            if !filtered.is_empty() {
                println!();
                for (i, cmd) in filtered.iter().enumerate() {
                    let is_selected = i == self.selected_command;
                    
                    // Two columns: command name (left), description (right)
                    if is_selected {
                        // Highlighted row - cyan background
                        print!("\x1B[46;30m");
                        print!("  /{:<20}", cmd.name);
                        print!("{:<40}", cmd.description);
                        println!("\x1B[0m");
                    } else {
                        // Normal row
                        print!("  \x1B[36m/{}\x1B[0m", cmd.name);
                        let name_pad = 20_usize.saturating_sub(cmd.name.len() + 1);
                        print!("{:pad$}", "", pad = name_pad);
                        println!("\x1B[90m{}\x1B[0m", cmd.description);
                    }
                }
            }
        }
        
        // Bottom separator and shortcuts
        println!();
        println!("\x1B[90m{}\x1B[0m", "─".repeat(term_width as usize));
        println!("\x1B[1mCtrl+C\x1B[0m Exit  \x1B[90m·\x1B[0m  \x1B[1mCtrl+R\x1B[0m Expand recent");
        
        io::stdout().flush().unwrap();
    }

    fn redraw_input_section(&self) {
        // Move cursor to line 10 (after header), clear down
        print!("\x1B[10;1H\x1B[J");
        self.draw_input_area();
    }

    fn get_filtered_commands(&self, filter: &str) -> Vec<&SlashCommand> {
        self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.to_lowercase().contains(&filter.to_lowercase()))
            .collect()
    }

    fn cleanup(&self) {
        let _ = disable_raw_mode();
        let mut out = stdout();
        let _ = out.execute(Show);
        print!("\x1B[0m");
        println!();
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self) -> io::Result<Option<(String, String)>> {
        let mut out = stdout();
        let _ = out.execute(Hide);
        
        self.draw_full();
        
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
                            // Ctrl+C - Exit
                            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                                return Ok(None);
                            }
                            // Ctrl+R - Expand recent (placeholder)
                            (KeyCode::Char('r'), m) if m.contains(KeyModifiers::CONTROL) => {
                                // TODO: Implement recent expansion
                            }
                            // Escape - Close menu or exit
                            (KeyCode::Esc, _) => {
                                if self.show_menu {
                                    self.show_menu = false;
                                    self.input.clear();
                                    self.selected_command = 0;
                                    self.redraw_input_section();
                                } else {
                                    return Ok(None);
                                }
                            }
                            // Arrow Up
                            (KeyCode::Up, _) => {
                                if self.show_menu && self.selected_command > 0 {
                                    self.selected_command -= 1;
                                    self.redraw_input_section();
                                }
                            }
                            // Arrow Down
                            (KeyCode::Down, _) => {
                                if self.show_menu {
                                    let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                    let max = self.get_filtered_commands(filter).len().saturating_sub(1);
                                    if self.selected_command < max {
                                        self.selected_command += 1;
                                        self.redraw_input_section();
                                    }
                                }
                            }
                            // Enter - Select command or submit
                            (KeyCode::Enter, _) => {
                                if self.show_menu {
                                    let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                    let filtered = self.get_filtered_commands(filter);
                                    if let Some(cmd) = filtered.get(self.selected_command) {
                                        if cmd.name == "exit" {
                                            return Ok(None);
                                        }
                                        if cmd.name == "clear" {
                                            self.input.clear();
                                            self.show_menu = false;
                                            self.selected_command = 0;
                                            self.draw_full();
                                            continue;
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
                            // Tab - Autocomplete
                            (KeyCode::Tab, _) => {
                                if self.show_menu {
                                    let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                    let filtered = self.get_filtered_commands(filter);
                                    if let Some(cmd) = filtered.get(self.selected_command) {
                                        self.input = format!("/{}", cmd.name);
                                        self.show_menu = false;
                                        self.redraw_input_section();
                                    }
                                }
                            }
                            // Backspace
                            (KeyCode::Backspace, _) => {
                                if !self.input.is_empty() {
                                    self.input.pop();
                                    self.show_menu = self.input.starts_with('/');
                                    self.selected_command = 0;
                                    self.redraw_input_section();
                                }
                            }
                            // Regular character input
                            (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                                self.input.push(c);
                                if self.input == "/" || self.input.starts_with('/') {
                                    self.show_menu = true;
                                    self.selected_command = 0;
                                } else {
                                    self.show_menu = false;
                                }
                                self.redraw_input_section();
                            }
                            _ => {}
                        }
                    }
                    Event::Resize(_, _) => {
                        self.draw_full();
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
