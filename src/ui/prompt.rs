use crossterm::{
    cursor::{self, MoveToColumn},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
use std::io::{self, Write};
use colored::*;

use crate::utils::config;

#[derive(Clone)]
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct InteractivePrompt {
    input: String,
    cursor_pos: usize,
    commands: Vec<SlashCommand>,
    show_commands: bool,
    selected_command: usize,
}

impl InteractivePrompt {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            commands: vec![
                SlashCommand { name: "recall", description: "Search your command history" },
                SlashCommand { name: "fix", description: "Fix a terminal error" },
                SlashCommand { name: "import", description: "Import shell history" },
                SlashCommand { name: "init", description: "Initialize Aethr" },
                SlashCommand { name: "status", description: "Check Aethr status" },
                SlashCommand { name: "login", description: "Authenticate" },
                SlashCommand { name: "help", description: "Show help" },
                SlashCommand { name: "exit", description: "Exit Aethr" },
            ],
            show_commands: false,
            selected_command: 0,
        }
    }

    fn get_separator(&self) -> String {
        let width = size().map(|(w, _)| w as usize).unwrap_or(80);
        "-".repeat(width.min(100))
    }

    fn print_welcome(&self) {
        let version = env!("CARGO_PKG_VERSION");
        
        println!();
        println!(" {}", "Welcome to Aethr".bold());
        println!(" Version {}", version);
        println!();
        println!(" Aethr recalls past commands and fixes terminal errors.");
        println!(" Type a task to get started or enter {} for commands.", "/".cyan());
        println!();
        
        // Login status
        let token_path = config::get_token_path();
        if token_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&token_path) {
                let content = content.trim();
                if !content.is_empty() && !content.starts_with('#') {
                    let display = if content.len() > 12 {
                        format!("{}...{}", &content[..4], &content[content.len()-4..])
                    } else {
                        "****".to_string()
                    };
                    println!(" {} Logged in: {}", "*".green(), display);
                } else {
                    println!(" {} Not logged in", "*".yellow());
                }
            } else {
                println!(" {} Not logged in", "*".yellow());
            }
        } else {
            println!(" {} Not logged in", "*".yellow());
        }
        println!();
    }

    fn print_input_box(&self) {
        let sep = self.get_separator();
        println!(" {}", sep.dimmed());
        print!(" {}  ", ">".cyan());
        println!("{}", "Enter / for commands".dimmed());
        println!(" {}", sep.dimmed());
        println!(" {}  {}",  "Ctrl+c".dimmed(), "Exit".dimmed());
    }

    fn draw_input(&self) -> io::Result<()> {
        execute!(io::stdout(), MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        print!(" {}  {}", ">".cyan(), self.input);
        io::stdout().flush()?;
        Ok(())
    }

    fn draw_commands(&self, filter: &str) -> io::Result<()> {
        let filtered: Vec<_> = self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.starts_with(filter))
            .collect();

        if filtered.is_empty() {
            return Ok(());
        }

        println!();
        
        for (idx, cmd) in filtered.iter().enumerate() {
            let is_selected = idx == self.selected_command;
            
            if is_selected {
                println!("    {} {:<12} {}", 
                    ">".cyan(),
                    format!("/{}", cmd.name).cyan().bold(),
                    cmd.description.dimmed()
                );
            } else {
                println!("      {:<12} {}", 
                    format!("/{}", cmd.name),
                    cmd.description.dimmed()
                );
            }
        }
        
        Ok(())
    }

    fn get_filtered_commands(&self, filter: &str) -> Vec<&SlashCommand> {
        self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.starts_with(filter))
            .collect()
    }

    pub fn run(&mut self) -> io::Result<Option<(String, String)>> {
        self.print_welcome();
        self.print_input_box();
        
        // Move cursor up to the input line (between the separators)
        execute!(io::stdout(), cursor::MoveUp(2))?;
        
        enable_raw_mode()?;
        self.draw_input()?;

        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            disable_raw_mode()?;
                            println!();
                            println!();
                            return Ok(None);
                        }
                        KeyCode::Esc => {
                            if self.show_commands {
                                self.show_commands = false;
                                self.redraw_full()?;
                            } else {
                                disable_raw_mode()?;
                                println!();
                                println!();
                                return Ok(None);
                            }
                        }
                        KeyCode::Enter => {
                            if self.show_commands && self.input.starts_with('/') {
                                let filter = &self.input[1..];
                                let filtered = self.get_filtered_commands(filter);
                                
                                if let Some(cmd) = filtered.get(self.selected_command) {
                                    disable_raw_mode()?;
                                    println!();
                                    println!();
                                    return Ok(Some((cmd.name.to_string(), String::new())));
                                }
                            } else if !self.input.is_empty() {
                                let input = self.input.clone();
                                disable_raw_mode()?;
                                println!();
                                println!();
                                
                                if input.starts_with('/') {
                                    let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
                                    let cmd = parts[0].to_string();
                                    let args = parts.get(1).unwrap_or(&"").to_string();
                                    return Ok(Some((cmd, args)));
                                } else {
                                    return Ok(Some(("query".to_string(), input)));
                                }
                            }
                        }
                        KeyCode::Up => {
                            if self.show_commands && self.selected_command > 0 {
                                self.selected_command -= 1;
                                self.redraw_full()?;
                            }
                        }
                        KeyCode::Down => {
                            if self.show_commands {
                                let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
                                let filtered = self.get_filtered_commands(filter);
                                if self.selected_command < filtered.len().saturating_sub(1) {
                                    self.selected_command += 1;
                                    self.redraw_full()?;
                                }
                            }
                        }
                        KeyCode::Tab => {
                            if self.show_commands && self.input.starts_with('/') {
                                let filter = &self.input[1..];
                                let filtered = self.get_filtered_commands(filter);
                                
                                if let Some(cmd) = filtered.get(self.selected_command) {
                                    self.input = format!("/{} ", cmd.name);
                                    self.cursor_pos = self.input.len();
                                    self.show_commands = false;
                                    self.redraw_full()?;
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.input.remove(self.cursor_pos);
                                self.show_commands = self.input.starts_with('/');
                                self.selected_command = 0;
                                self.redraw_full()?;
                            }
                        }
                        KeyCode::Char(c) => {
                            self.input.insert(self.cursor_pos, c);
                            self.cursor_pos += 1;
                            
                            if c == '/' && self.input == "/" {
                                self.show_commands = true;
                                self.selected_command = 0;
                            } else if self.input.starts_with('/') {
                                self.show_commands = true;
                            } else {
                                self.show_commands = false;
                            }
                            
                            self.redraw_full()?;
                        }
                        KeyCode::Left => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.draw_input()?;
                            }
                        }
                        KeyCode::Right => {
                            if self.cursor_pos < self.input.len() {
                                self.cursor_pos += 1;
                                self.draw_input()?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn redraw_full(&mut self) -> io::Result<()> {
        if self.show_commands {
            let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
            let filtered = self.get_filtered_commands(filter);
            let lines_to_clear = filtered.len() + 1;
            
            for _ in 0..lines_to_clear {
                execute!(io::stdout(), cursor::MoveUp(1), Clear(ClearType::CurrentLine))?;
            }
        }
        
        self.draw_input()?;
        
        if self.show_commands {
            let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
            
            disable_raw_mode()?;
            self.draw_commands(filter)?;
            enable_raw_mode()?;
            
            let filter_count = self.get_filtered_commands(filter).len();
            for _ in 0..filter_count {
                execute!(io::stdout(), cursor::MoveUp(1))?;
            }
        }
        
        Ok(())
    }
}

pub fn run_interactive() -> io::Result<Option<(String, String)>> {
    let mut prompt = InteractivePrompt::new();
    prompt.run()
}
