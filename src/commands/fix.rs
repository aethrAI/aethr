use crate::Result;
use crate::core::{fix_error, apply_rules_from_path};
use crate::db::moat::CommunityMoat;
use crate::utils::config;
use crate::ui::Status;
use colored::*;

pub fn run(error: &str) -> Result<()> {
    Status::pending(format!("Analyzing error: \"{}\"...", error.chars().take(40).collect::<String>()));
    println!();

    // 1) Try deterministic rules first (fastest, highest confidence)
    if let Ok(Some((cmd, conf, expl))) = apply_rules_from_path("data/error_rules.yaml", error) {
        Status::success("Rule match found");
        println!();
        println!("{}", "Command:".bright_black());
        println!("  {}", cmd.cyan());
        println!("{}", format!("Confidence: {:.0}%", conf * 100.0).bright_black());
        if !expl.is_empty() {
            println!("{}", "Explanation:".bright_black());
            println!("  {}", expl);
        }
        println!();
        println!("{}", "⚠️  Always review before running. Test in a safe environment first.".yellow());
        return Ok(());
    }

    // 2) Try community moat (high success entries from real developers)
    let db_path = config::get_db_path();
    if db_path.exists() {
        let moat = CommunityMoat::new(&db_path)?;
        // Search moat for entries that mention key tokens from the error
        let tokens: Vec<&str> = error
            .split_whitespace()
            .take(6)
            .filter(|t| !t.contains('\'') && !t.contains('"'))
            .collect();
        let query = tokens.join(" ");
        if !query.is_empty() {
            let results = moat.search(&query, None, 5).unwrap_or_default();
            if !results.is_empty() {
                Status::success("Found community solutions");
                println!();
                println!("{}", "Solutions from real developers:".bright_black());
                for (i, r) in results.iter().enumerate() {
                    println!("  {}. {}", i + 1, r.cyan());
                }
                println!();
                println!("{}", "⚠️  Review the suggestion and adapt to your environment.".yellow());
                return Ok(());
            }
        }
    }

    // 3) Fallback to LLM (if configured)
    Status::pending("Checking community database...");
    println!("\r{}", "No community match found. Attempting LLM analysis...".bright_black());
    println!();
    
    match fix_error(error) {
        Ok(suggestion) => {
            if let Some(cmd) = suggestion.command {
                Status::success("LLM suggestion ready");
                println!();
                println!("{}", "Command:".bright_black());
                println!("  {}", cmd.cyan());
                if let Some(conf) = suggestion.confidence {
                    println!("{}", format!("Confidence: {:.0}%", conf * 100.0).bright_black());
                }
                if let Some(expl) = suggestion.explanation {
                    println!("{}", "Explanation:".bright_black());
                    println!("  {}", expl);
                }
            } else {
                Status::success("LLM analysis complete");
                println!();
                if let Some(expl) = suggestion.explanation {
                    println!("{}", expl);
                }
            }
            println!();
            println!("{}", "⚠️  LLM suggestions are experimental. Always test before running.".yellow());
        }
        Err(e) => {
            Status::error(format!("LLM Error: {}", e));
            println!();
            println!("{}","💡 Troubleshooting:".yellow());
            println!("   1. Set ANTHROPIC_API_KEY environment variable for LLM features");
            println!("   2. Check your API key on console.anthropic.com");
            println!("   3. Try searching instead: {}", format!("aethr recall \"{}\"", error).cyan());
        }
    }
    Ok(())
}