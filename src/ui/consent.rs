use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use colored::*;

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
                ("Enable auto-save (local only)", "Commands saved privately on your machine", AutoSaveChoice::LocalOnly),
                ("Enable auto-save + share to Community Brain", "Help improve Aethr for everyone", AutoSaveChoice::ShareToCommunity),
                ("Disable auto-save", "You can import history manually later", AutoSaveChoice::Disabled),
            ],
        }
    }

    fn draw(&self) -> io::Result<()> {
        // Clear and draw options
        for (idx, (label, description, _)) in self.options.iter().enumerate() {
            execute!(io::stdout(), cursor::MoveToColumn(0), Clear(ClearType::CurrentLine))?;
            
            if idx == self.selected {
                println!("   {} {}", ">".cyan(), label.cyan().bold());
                println!("     {}", description.dimmed());
            } else {
                println!("     {}", label);
                println!("     {}", description.dimmed());
            }
        }
        
        io::stdout().flush()?;
        Ok(())
    }

    fn move_up(&self, lines: usize) -> io::Result<()> {
        for _ in 0..lines {
            execute!(io::stdout(), cursor::MoveUp(1))?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<AutoSaveChoice> {
        println!();
        println!(" {}", "Aethr Setup".bold());
        println!();
        println!(" Aethr can automatically save your commands to improve recall and fix suggestions.");
        println!(" Your data stays on your machine unless you choose to share.");
        println!();
        
        // Initial draw
        self.draw()?;
        
        println!();
        println!(" {}", "Use arrow keys to select, Enter to confirm".dimmed());
        
        // Move cursor back up to options area
        self.move_up(self.options.len() * 2 + 2)?;
        
        enable_raw_mode()?;

        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            disable_raw_mode()?;
                            println!();
                            return Ok(AutoSaveChoice::Disabled);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.selected > 0 {
                                self.selected -= 1;
                                self.draw()?;
                                self.move_up(self.options.len() * 2)?;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.selected < self.options.len() - 1 {
                                self.selected += 1;
                                self.draw()?;
                                self.move_up(self.options.len() * 2)?;
                            }
                        }
                        KeyCode::Enter => {
                            disable_raw_mode()?;
                            // Move past the options
                            for _ in 0..self.options.len() * 2 + 2 {
                                println!();
                            }
                            return Ok(self.options[self.selected].2);
                        }
                        KeyCode::Esc => {
                            disable_raw_mode()?;
                            println!();
                            return Ok(AutoSaveChoice::Disabled);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn show_consent() -> io::Result<AutoSaveChoice> {
    let mut prompt = ConsentPrompt::new();
    prompt.run()
}
