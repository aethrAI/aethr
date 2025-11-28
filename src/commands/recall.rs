use crate::Result;
use crate::db::local::LocalDB;
use crate::context::detect_project_context;
use crate::utils::config;
use colored::*;
use std::env;

pub fn run(query: &str) -> Result<()> {
    println!();
    
    let db_path = config::get_db_path();
    if !db_path.exists() {
        println!(" {} No local database found", "-".yellow());
        println!();
        println!(" Run {} first, then {}", "aethr init".cyan(), "aethr import".cyan());
        return Ok(());
    }
    
    let db = LocalDB::new(&db_path)?;
    
    // Detect current project context
    let cwd = env::current_dir().ok();
    let context = cwd.as_ref()
        .and_then(|p| detect_project_context(p).ok())
        .unwrap_or_default();
    
    // Show intelligent context message
    if !context.tags.is_empty() {
        println!(" Analyzing your {} project for relevant commands...", context.tags.join("/"));
    } else {
        println!(" Searching your command history...");
    }
    println!();
    
    // Search local history
    let local_scored = db.search_with_scores(query, 10).unwrap_or_default();
    
    // Apply context-aware boosting
    let mut results: Vec<(String, f32, bool, i64)> = Vec::new();
    
    for scored_cmd in local_scored {
        let context_boost = context.get_boost_multiplier(&scored_cmd.command);
        let final_score = scored_cmd.combined_score * context_boost;
        results.push((
            scored_cmd.command,
            final_score,
            context_boost > 1.0,
            scored_cmd.frequency,
        ));
    }
    
    // Sort by score
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Remove duplicates
    let mut seen = std::collections::HashSet::new();
    results.retain(|(cmd, _, _, _)| seen.insert(cmd.clone()));
    
    if results.is_empty() {
        println!(" {} No matches found for \"{}\"", "-".yellow(), query);
        println!();
        println!(" Try different keywords or run {} to add more history", "aethr import".cyan());
    } else {
        let boosted_count = results.iter().filter(|(_, _, boosted, _)| *boosted).count();
        if boosted_count > 0 {
            println!(" Found {} commands matching \"{}\", {} boosted by your current project:",
                results.len().min(10).to_string().cyan(),
                query,
                boosted_count);
        } else {
            println!(" Found {} commands matching \"{}\":",
                results.len().min(10).to_string().cyan(),
                query);
        }
        println!();
        
        for (i, (cmd, _score, is_boosted, frequency)) in results.iter().enumerate().take(10) {
            let boost = if *is_boosted { "*" } else { " " };
            let freq = format!("({}x)", frequency).dimmed();
            println!("  {}{} {}  {}", boost, i + 1, cmd.cyan(), freq);
        }
        
        if boosted_count > 0 {
            println!();
            println!(" {} = relevant to current project", "*".dimmed());
        }
    }
    println!();
    Ok(())
}
