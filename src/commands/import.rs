use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config;
use crate::ui::Spinner;
use colored::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run() -> Result<()> {
    let db_path = config::get_db_path();
    if !db_path.exists() {
        println!(" {} Run {} first.", "!".yellow(), "aethr init".cyan());
        return Ok(());
    }

    let db = LocalDB::new(&db_path)?;
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    
    let mut total_imported = 0;
    
    // Try to import from shell history files
    let mut history_files: Vec<(PathBuf, &str)> = vec![
        (home.join(".bash_history"), "bash"),
        (home.join(".zsh_history"), "zsh"),
    ];
    
    // Add platform-specific paths
    #[cfg(unix)]
    history_files.push((home.join(".local/share/fish/fish_history"), "fish"));
    
    // Git Bash on Windows uses different paths
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            history_files.push((PathBuf::from(&appdata).join("Microsoft\\Windows\\PowerShell\\PSReadLine\\ConsoleHost_history.txt"), "powershell"));
        }
    }
    
    println!();
    
    for (path, shell) in &history_files {
        if path.exists() {
            let spinner = Spinner::start(format!("Importing {} history...", shell));
            
            match import_history_file(&db, path, shell) {
                Ok(count) => {
                    if count > 0 {
                        spinner.success(format!("Imported {} commands from {}", count, shell));
                        total_imported += count;
                    } else {
                        spinner.info(format!("No new commands in {} history", shell));
                    }
                }
                Err(e) => {
                    spinner.error(format!("Failed to import {}: {}", shell, e));
                }
            }
        }
    }
    
    // Also process any pending commands.log
    let log_path = config::get_cmd_log_path();
    if log_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&log_path) {
            if !content.trim().is_empty() {
                let spinner = Spinner::start("Processing pending commands...");
                
                let mut count = 0;
                for line in content.lines() {
                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() == 3 {
                        let timestamp: i64 = parts[0].parse().unwrap_or(0);
                        let dir = parts[1];
                        let cmd = parts[2].trim();
                        if !cmd.is_empty() && timestamp > 0 {
                            if db.insert_command(cmd, dir, 0, timestamp).is_ok() {
                                count += 1;
                            }
                        }
                    }
                }
                
                if count > 0 {
                    spinner.success(format!("Imported {} pending commands", count));
                    total_imported += count;
                } else {
                    spinner.stop();
                }
                
                // Clear the log
                std::fs::write(&log_path, "")?;
            }
        }
    }
    
    println!();
    if total_imported == 0 {
        println!(" {} No new commands to import", "-".yellow());
        println!();
        println!(" To start auto-saving commands, run:");
        println!("   {}", "aethr hook --install".cyan());
    } else {
        println!(" {} Total: {} commands imported", "+".green(), total_imported);
    }
    
    println!();
    Ok(())
}

fn import_history_file(db: &LocalDB, path: &PathBuf, shell: &str) -> Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // Collect all commands first
    let mut commands: Vec<(String, String, i32, i64)> = Vec::new();
    let mut idx = 0;
    
    for line in reader.lines() {
        let line = line?;
        let cmd = parse_history_line(&line, shell);
        
        if let Some(cmd) = cmd {
            if !cmd.is_empty() && !should_skip(&cmd) {
                // Use decreasing timestamps so older commands have older timestamps
                let ts = now - (idx as i64);
                commands.push((cmd, "~".to_string(), 0, ts));
                idx += 1;
            }
        }
    }
    
    // Batch insert (much faster)
    if commands.is_empty() {
        return Ok(0);
    }
    
    db.insert_commands_batch(&commands)
}

fn parse_history_line(line: &str, shell: &str) -> Option<String> {
    match shell {
        "zsh" => {
            // Zsh history format: : timestamp:0;command
            if line.starts_with(": ") {
                line.splitn(2, ';').nth(1).map(|s| s.to_string())
            } else {
                Some(line.to_string())
            }
        }
        "fish" => {
            // Fish history format: - cmd: command
            if line.starts_with("- cmd: ") {
                Some(line[7..].to_string())
            } else {
                None
            }
        }
        _ => {
            // Bash: plain text, one command per line
            Some(line.to_string())
        }
    }
}

fn should_skip(cmd: &str) -> bool {
    let cmd = cmd.trim();
    
    // Skip empty or very short commands
    if cmd.len() < 2 {
        return true;
    }
    
    // Skip common noise
    let skip_prefixes = ["#", "cd ", "ls", "pwd", "clear", "exit", "aethr"];
    for prefix in &skip_prefixes {
        if cmd.starts_with(prefix) {
            return true;
        }
    }
    
    false
}