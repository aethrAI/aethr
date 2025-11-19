use crate::Result;
use crate::context::detect_project_context;
use crate::utils::api_client;
use colored::*;
use std::env;

pub fn run(prompt: &str) -> Result<()> {
    println!("{}", "🤖 Generating prediction...".yellow());
    
    // Detect current project context
    let cwd = env::current_dir().ok();
    let context = cwd.as_ref()
        .and_then(|p| detect_project_context(p).ok())
        .unwrap_or_default();
    
    // Show detected context if any tags found
    if !context.tags.is_empty() {
        println!("{}", format!("📍 Detected context: {}", context.tags.join(", ")).bright_black());
    }
    
    // Enhance the prompt with detected context
    let enhanced_prompt = if !context.tags.is_empty() {
        format!(
            "You are Aethr, a terminal assistant. The user is in a {} project. \
             Suggest a shell command for: {}",
            context.tags.join(", "),
            prompt
        )
    } else {
        format!("You are Aethr. Suggest a shell command for: {}", prompt)
    };
    
    let resp = api_client::call_claude(&enhanced_prompt)?;
    println!();
    println!("{}", "Prediction:".green());
    println!("{}", resp);
    Ok(())
}
