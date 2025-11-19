use crate::Result;
use crate::db::local::LocalDB;
use crate::db::moat::CommunityMoat;
use crate::context::detect_project_context;
use crate::utils::config;
use crate::ui::Status;
use colored::*;
use std::env;

pub fn run(query: &str) -> Result<()> {
    Status::pending(format!("Searching for \"{}\"...", query));
    
    let db_path = config::get_db_path();
    if !db_path.exists() {
        Status::error("No local database found");
        println!();
        println!("{}","📝 First steps:".yellow());
        println!("   1. Run: {}", "aethr init".cyan());
        println!("   2. Run: {}", "aethr import".cyan());
        println!("   3. Try again: {}", format!("aethr recall \"{}\"", query).cyan());
        return Ok(());
    }
    let db = LocalDB::new(&db_path)?;
    let moat = CommunityMoat::new(&db_path)?;
    
    // Detect current project context (current working directory)
    let cwd = env::current_dir().ok();
    let context = cwd.as_ref()
        .and_then(|p| detect_project_context(p).ok())
        .unwrap_or_default();
    
    // Show detected context if any tags found
    if !context.tags.is_empty() {
        println!("\r{}", format!("📍 Detected context: {}", context.tags.join(", ")).bright_black());
    }
    
    // Search local history with scoring (recency + frequency)
    let local_scored = db.search_with_scores(query, 10).unwrap_or_default();
    
    // Also search community moat
    let moat_results = moat.search(query, None, 5).unwrap_or_default();
    
    // Combine and apply context-aware boosting
    let mut all_results: Vec<(String, f32, bool, Option<i64>)> = Vec::new();
    
    // Add local results with their scores
    for scored_cmd in local_scored {
        let context_boost = context.get_boost_multiplier(&scored_cmd.command);
        let final_score = scored_cmd.combined_score * context_boost;
        all_results.push((
            scored_cmd.command,
            final_score,
            context_boost > 1.0,
            Some(scored_cmd.frequency),
        ));
    }
    
    // Add moat results with context boosting (no frequency since not personal history)
    for cmd in moat_results {
        let context_boost = context.get_boost_multiplier(&cmd);
        let moat_base_score = 0.75; // Moat results get baseline score
        let final_score = moat_base_score * context_boost;
        all_results.push((cmd, final_score, context_boost > 1.0, None));
    }
    
    // Remove duplicates, keeping the highest-scoring version
    all_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut seen = std::collections::HashSet::new();
    all_results.retain(|(cmd, _, _, _)| seen.insert(cmd.clone()));
    
    if all_results.is_empty() {
        println!("\r{}", "No matches found in your history or the community moat.".yellow());
        println!();
        println!("{}","💡 Tips:".yellow());
        println!("   • Try simpler keywords (e.g., 'docker' instead of 'docker container ls advanced')");
        println!("   • Run 'aethr seed-moat' to load 50+ community fixes");
        println!("   • Your personal history will be indexed after 'aethr import'");
    } else {
        println!();
        println!("{}", "📋 Top matches (your history + community):".green().bold());
        for (i, (cmd, _score, is_boosted, frequency)) in all_results.iter().enumerate().take(10) {
            let boost_indicator = if *is_boosted { "⭐ " } else { "" };
            let freq_indicator = if let Some(freq) = frequency {
                format!(" [used {} times]", freq).bright_black().to_string()
            } else {
                String::new()
            };
            println!("{}{}. {}{}", boost_indicator, i + 1, cmd.cyan(), freq_indicator);
        }
        println!();
        println!("{}", "💡 Tip: Use 'aethr fix' to get help with error messages.".bright_black());
    }
    Ok(())
}