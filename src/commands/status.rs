use crate::Result;
use crate::utils::config;
use std::fs;

pub fn run() -> Result<()> {
    let token_path = config::get_token_path();
    let token = fs::read_to_string(&token_path).unwrap_or_default();
    println!("Status (local):");
    println!("Token file: {}", token_path.display());
    if token.trim().is_empty() {
        println!("Token: <none set>");
    } else {
        println!("Token (first 8 chars): {}...", &token.trim()[..token.trim().len().min(8)]);
    }
    Ok(())
}
