use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    execute,
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
    selected_idx: usize,
    show_menu: bool,
    total_lines_drawn: u16,
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
            selected_idx: 0,
            show_menu: false,
            total_lines_drawn: 0,
        }
    }

    fn term_width() -> usize {
        size().map(|(w, _)| w as usize).unwrap_or(80)
    }

    fn filtered_commands(&self) -> Vec<&SlashCommand> {
        let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
        self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.to_lowercase().starts_with(&filter.to_lowercase()))
            .collect()
    }

    fn clear_drawn_lines(&self) {
        if self.total_lines_drawn > 0 {
            for _ in 0..self.total_lines_drawn {
                print!("\x1b[A\x1b[2K");
            }
            let _ = stdout().flush();
        }
    }

    fn draw(&mut self) {
        let w = Self::term_width();
        let mut lines: Vec<String> = Vec::new();
        
        lines.push(format!("\x1b[90m{}\x1b[0m", "━".repeat(w)));
        
        if self.input.is_empty() {
            lines.push(format!("> \x1b[7m \x1b[0m\x1b[90mEnter / for commands\x1b[0m"));
        } else {
            lines.push(format!("> {}\x1b[7m \x1b[0m", self.input));
        }
        
        if self.show_menu {
            let filtered = self.filtered_commands();
            if !filtered.is_empty() {
                for (i, cmd) in filtered.iter().enumerate() {
                    let is_sel = i == self.selected_idx;
                    if is_sel {
                        lines.push(format!("  \x1b[33m▶\x1b[0m \x1b[36m/{:<12}\x1b[0m \x1b[33m{}\x1b[0m", cmd.name, cmd.description));
                    } else {
                        lines.push(format!("    \x1b[36m/{:<12}\x1b[0m \x1b[90m{}\x1b[0m", cmd.name, cmd.description));
                    }
                }
            }
        }
        
        lines.push(format!("\x1b[90m{}\x1b[0m", "━".repeat(w)));
        lines.push(format!("\x1b[1mCtrl+C\x1b[0m Exit  \x1b[90m·\x1b[0m  \x1b[1m↑↓\x1b[0m Navigate  \x1b[90m·\x1b[0m  \x1b[1mEnter\x1b[0m Select"));
        
        self.clear_drawn_lines();
        
        for line in &lines {
            println!("{}", line);
        }
        let _ = stdout().flush();
        
        self.total_lines_drawn = lines.len() as u16;
    }

    fn print_header(&self) {
        let w = Self::term_width();
        let version = env!("CARGO_PKG_VERSION");
        
        println!();
        println!("\x1b[33mWelcome to Aethr CLI\x1b[0m");
        println!("Version {}", version);
        println!();
        println!("Aethr helps you recall, fix, and organize your terminal commands.");
        println!("Describe what you need or enter ? for help.");
        println!();
        
        let token_path = config::get_token_path();
        let has_token = token_path.exists() && 
            std::fs::read_to_string(&token_path).map(|s| !s.trim().is_empty()).unwrap_or(false);
        
        let username = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
        
        if has_token && !username.is_empty() {
            println!("\x1b[33m●\x1b[0m Logged in as: \x1b[1m{}\x1b[0m", username);
        }
        
        let cfg = AethrConfig::load();
        if cfg.auto_save {
            println!("\x1b[33m●\x1b[0m Auto-save enabled");
        }
        
        println!();
        
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("~"));
        let model = "claude-sonnet-4.5 (1x)";
        let available = w.saturating_sub(model.len() + 2);
        let path = if cwd.len() > available {
            format!("...{}", &cwd[cwd.len().saturating_sub(available - 3)..])
        } else {
            cwd
        };
        let gap = w.saturating_sub(path.len() + model.len());
        println!("{}{}\x1b[90m{}\x1b[0m", path, " ".repeat(gap), model);
    }

    pub fn run(&mut self) -> io::Result<Option<(String, String)>> {
        self.print_header();
        self.draw();
        
        execute!(stdout(), cursor::Hide)?;
        enable_raw_mode()?;
        
        let result = self.event_loop();
        
        disable_raw_mode()?;
        execute!(stdout(), cursor::Show)?;
        println!();
        
        result
    }

    fn event_loop(&mut self) -> io::Result<Option<(String, String)>> {
        loop {
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                            return Ok(None);
                        }
                        (KeyCode::Esc, _) => {
                            if self.show_menu {
                                self.input.clear();
                                self.show_menu = false;
                                self.selected_idx = 0;
                                disable_raw_mode()?;
                                self.draw();
                                enable_raw_mode()?;
                            } else {
                                return Ok(None);
                            }
                        }
                        (KeyCode::Up, _) if self.show_menu => {
                            if self.selected_idx > 0 {
                                self.selected_idx -= 1;
                                disable_raw_mode()?;
                                self.draw();
                                enable_raw_mode()?;
                            }
                        }
                        (KeyCode::Down, _) if self.show_menu => {
                            let max = self.filtered_commands().len().saturating_sub(1);
                            if self.selected_idx < max {
                                self.selected_idx += 1;
                                disable_raw_mode()?;
                                self.draw();
                                enable_raw_mode()?;
                            }
                        }
                        (KeyCode::Enter, _) => {
                            if self.show_menu {
                                let cmds = self.filtered_commands();
                                if let Some(cmd) = cmds.get(self.selected_idx) {
                                    let name = cmd.name.to_string();
                                    if name == "exit" {
                                        return Ok(None);
                                    }
                                    self.clear_drawn_lines();
                                    return Ok(Some((name, String::new())));
                                }
                            } else if !self.input.is_empty() {
                                let input = self.input.clone();
                                self.clear_drawn_lines();
                                if input.starts_with('/') {
                                    return Ok(Some((input[1..].to_string(), String::new())));
                                } else {
                                    return Ok(Some(("query".to_string(), input)));
                                }
                            }
                        }
                        (KeyCode::Tab, _) if self.show_menu => {
                            let cmds = self.filtered_commands();
                            if let Some(cmd) = cmds.get(self.selected_idx) {
                                self.input = format!("/{}", cmd.name);
                                self.show_menu = false;
                                self.selected_idx = 0;
                                disable_raw_mode()?;
                                self.draw();
                                enable_raw_mode()?;
                            }
                        }
                        (KeyCode::Backspace, _) => {
                            if !self.input.is_empty() {
                                self.input.pop();
                                self.show_menu = self.input.starts_with('/');
                                self.selected_idx = 0;
                                disable_raw_mode()?;
                                self.draw();
                                enable_raw_mode()?;
                            }
                        }
                        (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                            self.input.push(c);
                            self.show_menu = self.input.starts_with('/');
                            self.selected_idx = 0;
                            disable_raw_mode()?;
                            self.draw();
                            enable_raw_mode()?;
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
