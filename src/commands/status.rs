use crate::Result;
use crate::utils::config::{self, AethrConfig};
use crate::db::local::LocalDB;
use crate::db::community_brain::CommunityBrain;
use colored::*;
use std::fs;

pub fn run() -> Result<()> {
    println!();
    println!("{}", "Aethr Status".bold());
    println!();
    
    // Config status
    let config = AethrConfig::load();
    let config_dir = config::config_dir();
    
    println!("  {}", "Configuration".dimmed());
    println!("    Config dir:    {}", config_dir.display());
    
    // Auto-save status
    let auto_save_status = if config.auto_save {
        if config.share_to_community {
            "enabled + Community Brain".green()
        } else {
            "enabled (local only)".green()
        }
    } else {
        "disabled".yellow()
    };
    println!("    Auto-save:     {}", auto_save_status);
    
    // Shell hook status
    let hook_status = if config.shell_hook_installed {
        "installed".green()
    } else {
        "not installed".yellow()
    };
    println!("    Shell hook:    {}", hook_status);
    
    // Database status
    let db_path = config::get_db_path();
    println!();
    println!("  {}", "Database".dimmed());
    println!("    Path:          {}", db_path.display());
    
    if db_path.exists() {
        if let Ok(db) = LocalDB::new(&db_path) {
            if let Ok(count) = db.get_command_count() {
                println!("    Local history: {} commands", count.to_string().cyan());
            }
        }
        
        if let Ok(brain) = CommunityBrain::new(&db_path) {
            if let Ok(count) = brain.count() {
                println!("    Community Brain: {} fixes", count.to_string().cyan());
            }
        }
    } else {
        println!("    Status:        {}", "not initialized".yellow());
    }
    
    // Authentication status
    let token_path = config::get_token_path();
    let token = fs::read_to_string(&token_path).unwrap_or_default();
    println!();
    println!("  {}", "Authentication".dimmed());
    if token.trim().is_empty() {
        println!("    Status:        {}", "not logged in".yellow());
    } else {
        println!("    Status:        {}", "logged in".green());
        println!("    Token:         {}...", &token.trim()[..token.trim().len().min(8)]);
    }
    
    // Pending auto-save commands
    let log_path = config::get_cmd_log_path();
    if log_path.exists() {
        if let Ok(content) = fs::read_to_string(&log_path) {
            let pending = content.lines().count();
            if pending > 0 {
                println!();
                println!("  {}", "Pending".dimmed());
                println!("    Queued cmds:   {}", pending.to_string().cyan());
            }
        }
    }
    
    println!();
    
    Ok(())
}
