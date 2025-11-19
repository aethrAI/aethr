use crate::Result;
use crate::utils::config;
use std::fs;

pub fn run(token: &str) -> Result<()> {
    let token_path = config::get_token_path();
    std::fs::create_dir_all(config::config_dir())?;
    fs::write(&token_path, format!("{}\n", token.trim()))?;
    println!("Saved token to {}", token_path.display());
    Ok(())
}
