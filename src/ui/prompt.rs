use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, ClearType},
    execute, queue,
};
use std::io::{self, Write, stdout, Stdout};

use crate::utils::config::{self, AethrConfig};

#[derive(Clone)]
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct InteractivePrompt {
    input: String,
    cursor_pos: usize,
    commands: Vec<SlashCommand>,
    selected_idx: usize,
    show_menu: bool,
    start_row: u16,
}

impl InteractivePrompt {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            commands: vec![
                SlashCommand { name: "recall", description: "Search command history" },
                SlashCommand { name: "fix", description: "Fix a terminal error" },
                SlashCommand { name: "import", description: "Import shell history" },
                SlashCommand { name: "init", description: "Initialize Aethr" },
                SlashCommand { name: "status", description: "Check Aethr status" },
                SlashCommand { name: "clear", description: "Clear the screen" },
                SlashCommand { name: "help", description: "Show help for commands" },
                SlashCommand { name: "exit", description: "Exit Aethr" },
            ],
            selected_idx: 0,
            show_menu: false,
            start_row: 0,
        }
    }

    fn term_width() -> u16 {
        size().map(|(w, _)| w).unwrap_or(80)
    }

    fn print_static_header(&self) {
        let version = env!("CARGO_PKG_VERSION");
        let w = Self::term_width() as usize;
        
        // Yellow "Welcome" header
        println!("\x1b[33mWelcome to Aethr CLI\x1b[0m");
        println!("Version {}", version);
        println!();
        
        // Description text
        println!("Aethr helps you recall, fix, and organize your terminal commands. Describe");
        println!("what you need or enter ? for help. Aethr learns from your history.");
        println!();
        
        // Status bullets
        let token_path = config::get_token_path();
        let has_token = token_path.exists() && 
            std::fs::read_to_string(&token_path).map(|s| !s.trim().is_empty()).unwrap_or(false);
        
        let username = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
        
        if has_token {
            println!("\x1b[33m●\x1b[0m Logged in as user: \x1b[1m{}\x1b[0m", username);
        } else {
            println!("\x1b[90m○ Not logged in\x1b[0m");
        }
        
        let cfg = AethrConfig::load();
        if cfg.auto_save {
            println!("\x1b[33m●\x1b[0m Auto-save enabled");
        }
        
        println!();
        
        // Directory path and model badge on same line
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("~"));
        
        let model = "claude-sonnet-4.5 (1x)";
        let available = w.saturating_sub(model.len() + 2);
        let path_display = if cwd.len() > available {
            format!("...{}", &cwd[cwd.len() - available + 3..])
        } else {
            cwd
        };
        
        let gap = w.saturating_sub(path_display.len() + model.len());
        println!("{}{}\x1b[90m{}\x1b[0m", path_display, " ".repeat(gap), model);
        
        // Thick top separator
        println!("\x1b[90m{}\x1b[0m", "━".repeat(w));
    }

    fn render_input_line(&self, stdout: &mut Stdout) -> io::Result<()> {
        let w = Self::term_width() as usize;
        
        // Clear current line and print prompt
        queue!(stdout, cursor::MoveToColumn(0), terminal::Clear(ClearType::CurrentLine))?;
        
        // Prompt character
        print!("> ");
        
        if self.input.is_empty() {
            // Placeholder with blinking cursor
            print!("\x1b[7m \x1b[0m\x1b[90mEnter / for commands\x1b[0m");
        } else {
            // Show input with cursor
            let (before, after) = self.input.split_at(self.cursor_pos.min(self.input.len()));
            print!("{}\x1b[7m", before);
            if after.is_empty() {
                print!(" ");
            } else {
                print!("{}", after.chars().next().unwrap_or(' '));
            }
            print!("\x1b[0m");
            if after.len() > 1 {
                print!("{}", &after[1..]);
            }
        }
        
        stdout.flush()
    }

    fn render_menu(&self, stdout: &mut Stdout) -> io::Result<u16> {
        if !self.show_menu {
            return Ok(0);
        }
        
        let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
        let filtered: Vec<_> = self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.to_lowercase().starts_with(&filter.to_lowercase()))
            .collect();
        
        if filtered.is_empty() {
            return Ok(0);
        }
        
        println!(); // New line after input
        
        for (i, cmd) in filtered.iter().enumerate() {
            let selected = i == self.selected_idx;
            
            if selected {
                // Yellow indicator for selected
                print!("\x1b[33m█\x1b[0m \x1b[36m/{:<18}\x1b[0m \x1b[33m{}\x1b[0m", cmd.name, cmd.description);
            } else {
                // Gray for unselected
                print!("\x1b[90m█\x1b[0m \x1b[36m/{:<18}\x1b[0m \x1b[90m{}\x1b[0m", cmd.name, cmd.description);
            }
            println!();
        }
        
        stdout.flush()?;
        Ok(filtered.len() as u16)
    }

    fn render_footer(&self) {
        let w = Self::term_width() as usize;
        
        // Thick bottom separator
        println!("\x1b[90m{}\x1b[0m", "━".repeat(w));
        
        // Shortcuts
        print!("\x1b[1mCtrl+C\x1b[0m Exit  \x1b[90m·\x1b[0m  \x1b[1mTab\x1b[0m Complete  \x1b[90m·\x1b[0m  \x1b[1m↑↓\x1b[0m Navigate");
        
        let _ = io::stdout().flush();
    }

    fn full_render(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        
        // Get cursor position before we start
        let (_, start_y) = cursor::position()?;
        self.start_row = start_y;
        
        self.print_static_header();
        self.render_input_line(&mut stdout)?;
        let menu_lines = self.render_menu(&mut stdout)?;
        
        // Add spacing before footer if no menu
        if menu_lines == 0 {
            println!();
        }
        
        self.render_footer();
        println!();
        
        Ok(())
    }

    fn refresh_dynamic(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        
        // Calculate how many lines to go back
        let old_menu_lines = if self.show_menu {
            let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
            self.commands.iter()
                .filter(|c| filter.is_empty() || c.name.to_lowercase().starts_with(&filter.to_lowercase()))
                .count() as u16
        } else {
            0
        };
        
        // Move cursor up to input line (input + menu + footer spacing + footer)
        let lines_up = 1 + old_menu_lines + 2;
        execute!(stdout, cursor::MoveUp(lines_up.max(3)))?;
        
        // Clear from here to end of screen
        execute!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
        
        // Re-render dynamic parts
        self.render_input_line(&mut stdout)?;
        let menu_lines = self.render_menu(&mut stdout)?;
        
        if menu_lines == 0 {
            println!();
        }
        
        self.render_footer();
        println!();
        
        Ok(())
    }

    fn filtered_commands(&self) -> Vec<&SlashCommand> {
        let filter = if self.input.starts_with('/') { &self.input[1..] } else { "" };
        self.commands.iter()
            .filter(|c| filter.is_empty() || c.name.to_lowercase().starts_with(&filter.to_lowercase()))
            .collect()
    }

    pub fn run(&mut self) -> io::Result<Option<(String, String)>> {
        // Initial render
        self.full_render()?;
        
        // Hide cursor
        let mut stdout = stdout();
        execute!(stdout, cursor::Hide)?;
        
        // Enable raw mode
        enable_raw_mode()?;
        
        let result = self.event_loop();
        
        // Cleanup
        disable_raw_mode()?;
        execute!(stdout, cursor::Show)?;
        
        result
    }

    fn event_loop(&mut self) -> io::Result<Option<(String, String)>> {
        loop {
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    // Only process key press, not release
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    
                    let handled = match (key.code, key.modifiers) {
                        // Exit
                        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                            return Ok(None);
                        }
                        (KeyCode::Esc, _) => {
                            if self.show_menu {
                                self.input.clear();
                                self.cursor_pos = 0;
                                self.show_menu = false;
                                self.selected_idx = 0;
                                true
                            } else {
                                return Ok(None);
                            }
                        }
                        
                        // Navigation
                        (KeyCode::Up, _) if self.show_menu => {
                            if self.selected_idx > 0 {
                                self.selected_idx -= 1;
                            }
                            true
                        }
                        (KeyCode::Down, _) if self.show_menu => {
                            let max = self.filtered_commands().len().saturating_sub(1);
                            if self.selected_idx < max {
                                self.selected_idx += 1;
                            }
                            true
                        }
                        
                        // Selection
                        (KeyCode::Enter, _) => {
                            if self.show_menu {
                                let cmds = self.filtered_commands();
                                if let Some(cmd) = cmds.get(self.selected_idx) {
                                    let name = cmd.name.to_string();
                                    if name == "exit" {
                                        return Ok(None);
                                    }
                                    if name == "clear" {
                                        // Clear screen and re-render
                                        execute!(stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                                        self.input.clear();
                                        self.cursor_pos = 0;
                                        self.show_menu = false;
                                        self.selected_idx = 0;
                                        disable_raw_mode()?;
                                        self.full_render()?;
                                        enable_raw_mode()?;
                                        continue;
                                    }
                                    return Ok(Some((name, String::new())));
                                }
                            } else if !self.input.is_empty() {
                                let input = self.input.clone();
                                if input.starts_with('/') {
                                    return Ok(Some((input[1..].to_string(), String::new())));
                                } else {
                                    return Ok(Some(("query".to_string(), input)));
                                }
                            }
                            false
                        }
                        
                        // Tab autocomplete
                        (KeyCode::Tab, _) if self.show_menu => {
                            let cmds = self.filtered_commands();
                            if let Some(cmd) = cmds.get(self.selected_idx) {
                                self.input = format!("/{}", cmd.name);
                                self.cursor_pos = self.input.len();
                                self.show_menu = false;
                                self.selected_idx = 0;
                            }
                            true
                        }
                        
                        // Text editing
                        (KeyCode::Backspace, _) => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.input.remove(self.cursor_pos);
                                self.show_menu = self.input.starts_with('/');
                                self.selected_idx = 0;
                            }
                            true
                        }
                        (KeyCode::Left, _) => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                            }
                            true
                        }
                        (KeyCode::Right, _) => {
                            if self.cursor_pos < self.input.len() {
                                self.cursor_pos += 1;
                            }
                            true
                        }
                        (KeyCode::Home, _) => {
                            self.cursor_pos = 0;
                            true
                        }
                        (KeyCode::End, _) => {
                            self.cursor_pos = self.input.len();
                            true
                        }
                        
                        // Character input
                        (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                            self.input.insert(self.cursor_pos, c);
                            self.cursor_pos += 1;
                            
                            self.show_menu = self.input.starts_with('/');
                            self.selected_idx = 0;
                            true
                        }
                        
                        _ => false,
                    };
                    
                    if handled {
                        // Temporarily disable raw mode for clean output
                        disable_raw_mode()?;
                        self.refresh_dynamic()?;
                        enable_raw_mode()?;
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
