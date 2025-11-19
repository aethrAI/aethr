use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config;
use chrono::Utc;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn run() -> Result<()> {
    let log_path = config::get_cmd_log_path();
    if !log_path.exists() {
        println!("No command log found at {}. If you want to log commands, add the shell hook to ~/.bashrc or ~/.zshrc.", log_path.display());
        return Ok(());
    }

    let db_path = config::get_db_path();
    if !db_path.exists() {
        println!("Local DB not found. Run `aethr init` first.");
        return Ok(());
    }

    let db = LocalDB::new(&db_path)?;
    let file = File::open(&log_path)?;
    let reader = BufReader::new(file);

    let mut inserted = 0usize;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let ts_part = parts.next().unwrap_or("");
        let cmd_part = parts.next().unwrap_or("").trim();
        if cmd_part.is_empty() {
            continue;
        }
        let timestamp: i64 = ts_part.parse().unwrap_or_else(|_| Utc::now().timestamp());
        db.insert_command(cmd_part, ".", 0, timestamp)?;
        inserted += 1;
    }

    println!("Imported {} commands from {}", inserted, log_path.display());
    Ok(())
}