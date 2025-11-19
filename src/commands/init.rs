use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config;
use crate::ui::Status;
use colored::*;

pub fn run() -> Result<()> {
    println!("{}", "╔══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                    AETHR V1.0                            ║".cyan());
    println!("{}", "║          Terminal Intelligence that learns               ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".cyan());
    println!();

    let config_dir = config::config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
        Status::success(format!("Created config directory: {}", config_dir.display()));
    } else {
        Status::info(format!("Config directory exists: {}", config_dir.display()));
    }

    let db_path = config::get_db_path();
    let _db = LocalDB::new(&db_path)?;
    Status::success(format!("Created local database: {}", db_path.display()));

    // Create empty token file if missing (placeholder)
    let token_path = config::get_token_path();
    if !token_path.exists() {
        std::fs::create_dir_all(config_dir.clone())?;
        std::fs::write(&token_path, "# Add your API token here\n")?;
        Status::success(format!("Created token file placeholder: {}", token_path.display()));
    }

    println!();
    println!("{}", "🎉 Aethr initialized successfully!".green().bold());
    println!();
    println!("{}", "📝 Next steps:".yellow());
    println!("   1. Add the shell hook to log commands (optional).");
    println!("   2. Run: {}", "aethr import".cyan());
    println!("   3. Run: {}", "aethr recall <query>".cyan());
    println!();
    Ok(())
}