use crate::Result;
use crate::core::apply_rules_from_path;
use crate::db::community_brain::CommunityBrain;
use crate::context::detector::detect_project_context;
use crate::llm::ClaudeClient;
use crate::utils::config;
use std::io::{self, Write};

fn term_width() -> usize {
    crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

fn separator() {
    let w = term_width();
    println!("\x1b[90m{}\x1b[0m", "━".repeat(w));
}

fn header(title: &str) {
    separator();
    println!("\x1b[1;33m{}\x1b[0m", title);
    separator();
}

pub fn run(error: &str) -> Result<()> {
    println!();
    
    // Analyzing header
    println!("\x1b[90m🔍 Analyzing error...\x1b[0m");
    println!();
    
    // Show truncated error
    let error_preview: String = error.chars().take(80).collect();
    println!("\x1b[90m\"{}\"\x1b[0m", error_preview);
    println!();

    // Detect current context
    let context = detect_project_context(&std::env::current_dir().unwrap_or_default()).unwrap_or_default();
    let context_str = if context.tags.is_empty() {
        None
    } else {
        Some(context.tags.join(","))
    };
    
    if !context.tags.is_empty() {
        println!("\x1b[90mContext: {}\x1b[0m", context.tags.join(", "));
        println!();
    }

    // Layer 1: Deterministic rules (fastest, highest confidence)
    if let Ok(Some((cmd, conf, expl))) = apply_rules_from_path("data/error_rules.yaml", error) {
        show_problem(error);
        show_fix_with_source(&cmd, Some(conf * 100.0), 0, &expl, "Rule Engine", None);
        prompt_feedback(&cmd, error, context_str.as_deref())?;
        return Ok(());
    }

    // Layer 2: Community Brain (shared knowledge from real devs)
    let db_path = config::get_db_path();
    
    // Create parent directory if needed
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    let brain = CommunityBrain::new(&db_path)?;
    
    // Seed with common fixes if empty (for new users)
    brain.seed_if_empty()?;
    
    // Extract meaningful tokens from error
    let stopwords = ["the", "and", "for", "not", "was", "error", "found", "command", "no", "such", "file", "cannot"];
    let tokens: Vec<&str> = error
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|t| t.len() > 2)
        .filter(|t| !stopwords.contains(&t.to_lowercase().as_str()))
        .take(5)
        .collect();
    
    let query = tokens.join(" ");
    if !query.is_empty() {
        let context_tags: Option<Vec<&str>> = context_str.as_ref().map(|s| s.split(',').collect());
        let results = brain.search_with_scores(&query, context_tags.as_deref(), 3)?;
        
        if !results.is_empty() {
            show_problem(error);
            
            // Primary fix (highest scored)
            let primary = &results[0];
            let uses = (primary.success_count + primary.fail_count) as i32;
            let dev_text = if uses == 1 { "dev" } else { "devs" };
            
            show_fix_with_source(
                &primary.command,
                Some(primary.success_rate),
                uses,
                "",
                "Community",
                Some(format!("Used by {} {}", uses, dev_text))
            );
            
            // Alternative fixes
            if results.len() > 1 {
                println!();
                println!("\x1b[90m💡 Alternative solutions:\x1b[0m");
                println!();
                
                for (i, r) in results.iter().skip(1).enumerate() {
                    let alt_uses = (r.success_count + r.fail_count) as i32;
                    println!("  \x1b[90m{}.\x1b[0m \x1b[36m{}\x1b[0m", i + 2, r.command);
                    println!("     \x1b[90m{:.0}% success · {} uses\x1b[0m", r.success_rate, alt_uses);
                    println!();
                }
            }
            
            prompt_feedback(&primary.command, error, context_str.as_deref())?;
            return Ok(());
        }
    }

    // Layer 3: LLM fallback (Claude)
    if let Some(client) = ClaudeClient::from_env() {
        println!("\x1b[90m⚡ Consulting AI...\x1b[0m");
        println!();
        
        match client.get_fix(error, context_str.as_deref()) {
            Ok(suggestion) => {
                if !suggestion.command.is_empty() {
                    show_problem(error);
                    show_fix_with_source(
                        &suggestion.command,
                        None,
                        0,
                        &suggestion.explanation,
                        "AI",
                        Some("Claude Sonnet 4".to_string())
                    );
                    
                    println!();
                    println!("\x1b[90m⚠️  AI-generated solution (not community-verified)\x1b[0m");
                    println!();
                    
                    prompt_feedback(&suggestion.command, error, context_str.as_deref())?;
                    return Ok(());
                }
            }
            Err(e) => {
                println!("\x1b[33m⚠️  AI unavailable: {}\x1b[0m", e.to_string().chars().take(50).collect::<String>());
                println!();
            }
        }
    }

    // No solution found
    header("NO FIX FOUND");
    println!();
    println!("We couldn't find a solution in:");
    println!("  • Rule engine");
    println!("  • Community Brain");
    println!("  • AI (Claude)");
    println!();
    
    println!("Try rephrasing or share this error with the community:");
    println!("  \x1b[36m$ aethr report \"your error\"\x1b[0m");
    println!();
    
    Ok(())
}

/// Display "THE PROBLEM" section
fn show_problem(error: &str) {
    header("THE PROBLEM");
    println!();
    
    // Show error in a readable format
    let lines: Vec<&str> = error.lines().take(5).collect();
    for line in lines {
        println!("  \x1b[31m{}\x1b[0m", line.chars().take(100).collect::<String>());
    }
    println!();
}

/// Display "THE FIX" section with source attribution
fn show_fix_with_source(
    cmd: &str,
    confidence: Option<f32>,
    uses: i32,
    explanation: &str,
    source: &str,
    source_detail: Option<String>
) {
    header("THE FIX");
    println!();
    
    // Source badge
    let badge = match source {
        "Community" => "\x1b[32m✓ COMMUNITY VERIFIED\x1b[0m",
        "Rule Engine" => "\x1b[36m⚡ RULE MATCH\x1b[0m",
        "AI" => "\x1b[33m🤖 AI GENERATED\x1b[0m",
        _ => source,
    };
    println!("  {}", badge);
    
    if let Some(detail) = source_detail {
        println!("  \x1b[90m{}\x1b[0m", detail);
    }
    println!();
    
    // Command
    println!("  \x1b[1;36m{}\x1b[0m", cmd);
    println!();
    
    // Confidence/Success metrics
    if let Some(conf) = confidence {
        if uses > 0 {
            let bar_len = 20;
            let filled = ((conf / 100.0) * bar_len as f32) as usize;
            let empty = bar_len - filled;
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
            
            let color = if conf >= 80.0 { "\x1b[32m" } // Green
                else if conf >= 50.0 { "\x1b[33m" } // Yellow
                else { "\x1b[31m" }; // Red
            
            println!("  {}🎯 {:.0}% Success Rate{} {}", color, conf, "\x1b[0m", format!("\x1b[90m{}\x1b[0m", bar));
        } else {
            println!("  \x1b[90m🎯 {:.0}% Confidence\x1b[0m", conf);
        }
        println!();
    }
    
    // Explanation
    if !explanation.is_empty() {
        println!("  \x1b[90m{}\x1b[0m", explanation);
        println!();
    }
}

/// Prompt user for feedback and log to Community Brain
fn prompt_feedback(command: &str, error: &str, context_tags: Option<&str>) -> Result<()> {
    separator();
    print!("\x1b[1mDid this fix work?\x1b[0m \x1b[90m[Y/n]\x1b[0m ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    let db_path = config::get_db_path();
    if db_path.exists() {
        let brain = CommunityBrain::new(&db_path)?;
        
        // Normalize error pattern
        let error_pattern: String = error
            .chars()
            .take(100)
            .collect::<String>()
            .split_whitespace()
            .take(10)
            .collect::<Vec<&str>>()
            .join(" ");
        
        if input.is_empty() || input == "y" || input == "yes" {
            brain.log_success(command, &error_pattern, context_tags)?;
            println!();
            println!("\x1b[32m✓ Thanks! Added to Community Brain\x1b[0m");
            println!("\x1b[90m  Your feedback helps other developers\x1b[0m");
        } else if input == "n" || input == "no" {
            brain.log_failure(command, &error_pattern)?;
            println!();
            println!("\x1b[33m○ Feedback recorded\x1b[0m");
            println!("\x1b[90m  We'll improve suggestions based on this\x1b[0m");
        }
    }
    
    println!();
    Ok(())
}
