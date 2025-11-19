use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub struct CommandMenu {
    commands: Vec<CommandOption>,
    selected: usize,
}

pub struct CommandOption {
    pub name: &'static str,
    pub description: &'static str,
    pub alias: Option<&'static str>,
    pub example: &'static str,
}

impl CommandMenu {
    pub fn new() -> Self {
        Self {
            commands: vec![
                CommandOption {
                    name: "Recall",
                    description: "Search your command history semantically",
                    alias: Some("recall"),
                    example: "aethr recall \"docker\"",
                },
                CommandOption {
                    name: "Fix",
                    description: "Get instant fixes for terminal errors",
                    alias: Some("fix"),
                    example: "aethr fix \"permission denied\"",
                },
                CommandOption {
                    name: "Predict",
                    description: "AI-powered command suggestions",
                    alias: Some("predict"),
                    example: "aethr predict \"deploy\"",
                },
                CommandOption {
                    name: "Import",
                    description: "Import your shell history into Aethr",
                    alias: Some("import"),
                    example: "aethr import",
                },
                CommandOption {
                    name: "Init",
                    description: "Initialize Aethr (create database)",
                    alias: Some("init"),
                    example: "aethr init",
                },
                CommandOption {
                    name: "Seed Moat",
                    description: "Load 54+ community fixes",
                    alias: Some("seed-moat"),
                    example: "aethr seed-moat",
                },
                CommandOption {
                    name: "Status",
                    description: "Check Aethr daemon status",
                    alias: Some("status"),
                    example: "aethr status",
                },
                CommandOption {
                    name: "Daemon Start",
                    description: "Start background sync daemon",
                    alias: Some("daemon-start"),
                    example: "aethr daemon-start",
                },
                CommandOption {
                    name: "Daemon Stop",
                    description: "Stop background sync daemon",
                    alias: Some("daemon-stop"),
                    example: "aethr daemon-stop",
                },
                CommandOption {
                    name: "Help",
                    description: "Show help information",
                    alias: Some("--help"),
                    example: "aethr --help",
                },
            ],
            selected: 0,
        }
    }

    pub fn run(&mut self) -> io::Result<Option<String>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Draw initial frame
        terminal.draw(|f| self.ui(f))?;

        loop {
            if event::poll(std::time::Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Up => {
                            self.selected = if self.selected > 0 {
                                self.selected - 1
                            } else {
                                self.commands.len() - 1
                            };
                        }
                        KeyCode::Down => {
                            self.selected = (self.selected + 1) % self.commands.len();
                        }
                        KeyCode::Enter => {
                            let cmd = self.commands[self.selected].alias.unwrap_or("run");
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen
                            )?;
                            terminal.show_cursor()?;
                            return Ok(Some(cmd.to_string()));
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen
                            )?;
                            terminal.show_cursor()?;
                            return Ok(None);
                        }
                        KeyCode::Char(c) => {
                            // Quick select by first letter
                            if let Some(idx) = self.commands.iter().position(|cmd| {
                                cmd.name.to_lowercase().starts_with(c.to_lowercase().to_string().as_str())
                            }) {
                                self.selected = idx;
                            }
                        }
                        _ => {}
                    }
                    terminal.draw(|f| self.ui(f))?;
                }
            }
        }
    }

    fn ui<B: ratatui::backend::Backend>(&self, f: &mut ratatui::Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(15),
                    Constraint::Length(4),
                ]
                .as_ref(),
            )
            .split(f.size());

        // Header
        let header = Paragraph::new("Aethr — Terminal Intelligence")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(header, chunks[0]);

        // Command list
        let mut lines = vec![];
        for (idx, cmd) in self.commands.iter().enumerate() {
            let is_selected = idx == self.selected;
            let style = if is_selected {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▸ " } else { "  " };
            let line = Spans::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{:<15}", cmd.name), style),
                Span::styled(" ", style),
                Span::styled(cmd.description, {
                    if is_selected {
                        Style::default().bg(Color::Cyan).fg(Color::Black)
                    } else {
                        Style::default().fg(Color::Gray)
                    }
                }),
            ]);
            lines.push(line);
        }

        let commands_block = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Commands")
                    .title_alignment(Alignment::Left),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(commands_block, chunks[1]);

        // Footer
        let selected_cmd = &self.commands[self.selected];
        let footer_text = format!(
            "↑/↓: Navigate  |  Enter: Select  |  q/Esc: Quit  |  Example: {}",
            selected_cmd.example
        );
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_creation() {
        let menu = CommandMenu::new();
        assert_eq!(menu.commands.len(), 10);
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn test_first_command() {
        let menu = CommandMenu::new();
        assert_eq!(menu.commands[0].name, "Recall");
    }
}
