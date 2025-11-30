use crate::Result;
use crate::core::apply_rules_from_path;
use crate::db::community_brain::CommunityBrain;
use crate::context::detector::detect_project_context;
use crate::llm::ClaudeClient;
use crate::utils::config;
use colored::*;
use std::io::{self, Write};

pub fn run(error: &str) -> Result<()> {
    println!();
    println!(" {} \"{}\"", "Analyzing:".dimmed(), error.chars().take(60).collect::<String>());
    println!();

    // Detect current context
    let context = detect_project_context(&std::env::current_dir().unwrap_or_default()).unwrap_or_default();
    let context_str = if context.tags.is_empty() {
        None
    } else {
        Some(context.tags.join(","))
    };

    // Layer 1: Deterministic rules (fastest, highest confidence)
    if let Ok(Some((cmd, conf, expl))) = apply_rules_from_path("data/error_rules.yaml", error) {
        println!(" {} Rule match", "Layer 1".cyan().bold());
        println!();
        show_fix(&cmd, Some(conf * 100.0), None, &expl);
        prompt_feedback(&cmd, error, context_str.as_deref())?;
        return Ok(());
    }

    // Layer 2: Local history (your past solutions)
    // TODO: Search local command history for similar error patterns

    // Layer 3: Community Brain (shared knowledge)
    let db_path = config::get_db_path();
    if db_path.exists() {
        let brain = CommunityBrain::new(&db_path)?;
        
        // Extract meaningful tokens from error (skip common words)
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
                // Show intelligent message based on result count
                let error_type = tokens.first().unwrap_or(&"this");
                println!(" Found {} related {} from developers who resolved {} issues:",
                    results.len().to_string().cyan(),
                    if results.len() == 1 { "fix" } else { "fixes" },
                    error_type);
                println!();
                
                for (i, r) in results.iter().enumerate() {
                    let uses = r.success_count + r.fail_count;
                    let rate_str = if uses > 0 {
                        format!("{:.0}% success rate ({} uses)", r.success_rate, uses)
                    } else {
                        "new suggestion".to_string()
                    };
                    
                    println!("  {} {}", format!("{}.", i + 1).dimmed(), r.command.cyan());
                    println!("    {}", rate_str.dimmed());
                    println!();
                }
                
                // If only one result or first has high score, suggest it
                if results.len() == 1 || results[0].success_rate >= 70.0 {
                    prompt_feedback(&results[0].command, error, context_str.as_deref())?;
                }
                
                return Ok(());
            }
        }
    }

    // Layer 4: LLM fallback (Claude)
    if let Some(client) = ClaudeClient::from_env() {
        println!(" {} Asking Claude...", "Layer 3".cyan().bold());
        println!();
        
        match client.get_fix(error, context_str.as_deref()) {
            Ok(suggestion) => {
                if !suggestion.command.is_empty() {
                    show_fix(&suggestion.command, None, None, &suggestion.explanation);
                    println!(" {} LLM suggestion", "Source:".dimmed());
                    println!();
                    prompt_feedback(&suggestion.command, error, context_str.as_deref())?;
                    return Ok(());
                }
            }
            Err(e) => {
                println!(" {} LLM error: {}", "!".yellow(), e.to_string().chars().take(50).collect::<String>());
                println!();
            }
        }
    }

    // No solution found
    println!(" {} No fix found", "x".red());
    println!();
    println!(" Searched: rules, Community Brain{}", 
        if ClaudeClient::from_env().is_some() { ", Claude" } else { "" });
    println!();
    if ClaudeClient::from_env().is_none() {
        println!(" {} Set ANTHROPIC_API_KEY for LLM suggestions", "Tip:".dimmed());
        println!();
    }
    
    Ok(())
}

/// Display a fix suggestion
fn show_fix(cmd: &str, confidence: Option<f32>, success_rate: Option<f32>, explanation: &str) {
    println!(" {}", "Suggested fix:".bold());
    println!();
    println!("   {}", cmd.cyan());
    println!();
    
    if let Some(conf) = confidence {
        println!(" {} {:.0}%", "Confidence:".dimmed(), conf);
    }
    if let Some(rate) = success_rate {
        println!(" {} {:.0}%", "Success rate:".dimmed(), rate);
    }
    if !explanation.is_empty() {
        println!();
        println!(" {}", explanation.dimmed());
    }
    println!();
}

/// Prompt user for feedback and log to Community Brain
fn prompt_feedback(command: &str, error: &str, context_tags: Option<&str>) -> Result<()> {
    print!(" Did this fix work? {} ", "[Y/n]".dimmed());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    let db_path = config::get_db_path();
    if db_path.exists() {
        let brain = CommunityBrain::new(&db_path)?;
        
        // Normalize error pattern (first 100 chars, simplified)
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
            println!(" {} Logged to Community Brain", "+".green());
        } else if input == "n" || input == "no" {
            brain.log_failure(command, &error_pattern)?;
            println!(" {} Feedback recorded", "-".yellow());
        }
    }
    
    println!();
    Ok(())
}
