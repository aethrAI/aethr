use crate::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Modifier, Style, Color},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Terminal,
};
use std::{io, time::Duration};
use crate::db::local::LocalDB;
use crate::context::detect_project_context;
use crate::utils::config;
use std::env;

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let db_path = config::get_db_path();
    let db = if db_path.exists() {
        LocalDB::new(&db_path).ok()
    } else {
        None
    };

    // Detect project context
    let cwd = env::current_dir().ok();
    let context = cwd.as_ref()
        .and_then(|p| detect_project_context(p).ok())
        .unwrap_or_default();

    let mut input = String::new();
    let mut results: Vec<(String, Option<i64>)> = Vec::new(); // (command, frequency)
    let mut selected = 0usize;
    let mut list_state = ListState::default();
    list_state.select(Some(selected));
    let mut status_message = String::new();
    let mut show_status = false;
    let mut status_timer = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(4),
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Header with context
            let context_str = if !context.tags.is_empty() {
                format!(" [{}]", context.tags.join(", "))
            } else {
                String::new()
            };
            let header_text = format!(
                "🔍 Aethr — Context-Gated Terminal Intelligence{}",
                context_str
            );
            let header = Paragraph::new(header_text)
                .block(Block::default().borders(Borders::ALL).title("Aethr TUI"))
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(header, chunks[0]);

            // Search input box with better styling
            let input_display = if input.is_empty() {
                "Type to search your command history...".to_string()
            } else {
                input.clone()
            };
            let input_para = Paragraph::new(input_display)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Search Query")
                        .border_style(if input.is_empty() {
                            Style::default().fg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Green)
                        })
                )
                .style(Style::default().fg(Color::White));
            f.render_widget(input_para, chunks[1]);

            // Results list with enhanced styling
            let items: Vec<ListItem> = results
                .iter()
                .enumerate()
                .map(|(idx, (cmd, freq))| {
                    let is_selected = idx == selected;
                    let context_boost = context.get_boost_multiplier(cmd);
                    let is_boosted = context_boost > 1.0;
                    
                    let freq_text = freq.map(|f| format!(" [×{}]", f)).unwrap_or_default();
                    let boost_indicator = if is_boosted { "⭐ " } else { "  " };
                    
                    let cmd_text = format!("{}{}{}", boost_indicator, cmd, freq_text);
                    
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else if is_boosted {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    ListItem::new(cmd_text).style(style)
                })
                .collect();
            
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Results ({})", results.len()))
                )
                .style(Style::default().fg(Color::White));
            f.render_stateful_widget(list, chunks[2], &mut list_state);

            // Footer with help text and status
            let footer_text = if show_status {
                status_message.clone()
            } else {
                "↑↓=move  Enter=copy  Ctrl+C=quit".to_string()
            };
            
            let footer = Paragraph::new(footer_text)
                .alignment(Alignment::Center)
                .style(if show_status {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .block(Block::default().borders(Borders::TOP));
            f.render_widget(footer, chunks[3]);
        })?;

        // Decrement status timer
        if show_status {
            status_timer -= 1;
            if status_timer <= 0 {
                show_status = false;
            }
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. }) => break,
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => break,
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                    input.push(c);
                    results = if let Some(ref db) = db {
                        db.search_with_scores(&input, 20)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|score| (score.command, Some(score.frequency)))
                            .collect()
                    } else {
                        vec![]
                    };
                    selected = 0;
                    list_state.select(Some(selected));
                }
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    input.pop();
                    results = if let Some(ref db) = db {
                        db.search_with_scores(&input, 20)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|score| (score.command, Some(score.frequency)))
                            .collect()
                    } else {
                        vec![]
                    };
                    selected = 0;
                    list_state.select(Some(selected));
                }
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    if selected > 0 {
                        selected -= 1;
                    }
                    list_state.select(Some(selected));
                }
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    selected = selected.saturating_add(1).min(results.len().saturating_sub(1));
                    list_state.select(Some(selected));
                }
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    if let Some((choice, _)) = results.get(selected) {
                        // Try to copy to clipboard using xclip or pbcopy
                        let copy_result = if cfg!(target_os = "linux") {
                            std::process::Command::new("xclip")
                                .arg("-selection")
                                .arg("clipboard")
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                                .ok()
                                .and_then(|mut child| {
                                    if let Some(mut stdin) = child.stdin.take() {
                                        use std::io::Write;
                                        stdin.write_all(choice.as_bytes()).ok()
                                    } else {
                                        None
                                    }
                                })
                                .is_some()
                        } else if cfg!(target_os = "macos") {
                            std::process::Command::new("pbcopy")
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                                .ok()
                                .and_then(|mut child| {
                                    if let Some(mut stdin) = child.stdin.take() {
                                        use std::io::Write;
                                        stdin.write_all(choice.as_bytes()).ok()
                                    } else {
                                        None
                                    }
                                })
                                .is_some()
                        } else {
                            false
                        };

                        disable_raw_mode()?;
                        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                        
                        println!("\n✅ Command: {}", choice);
                        if copy_result {
                            println!("📋 Copied to clipboard!");
                        } else {
                            println!("(Clipboard not available on this system)");
                        }
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
