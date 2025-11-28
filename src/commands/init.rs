use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config::{self, AethrConfig};
use crate::ui::{show_consent, AutoSaveChoice};
use colored::*;

pub fn run() -> Result<()> {
    let config_dir = config::config_dir();
    let config_file = config_dir.join("config.json");
    
    // Check if this is first run (no config file)
    let is_first_run = !config_file.exists();
    
    // Create config directory
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
        println!(" {} Created {}", "+".green(), config_dir.display());
    }

    // Initialize database
    let db_path = config::get_db_path();
    let _db = LocalDB::new(&db_path)?;
    println!(" {} Database ready", "+".green());

    // Create token file
    let token_path = config::get_token_path();
    if !token_path.exists() {
        std::fs::write(&token_path, "")?;
    }

    // On first run, show consent prompt
    if is_first_run {
        println!();
        
        match show_consent() {
            Ok(choice) => {
                let config = match choice {
                    AutoSaveChoice::LocalOnly => {
                        println!(" {} Auto-save enabled (local only)", "+".green());
                        AethrConfig {
                            auto_save: true,
                            share_to_community: false,
                            shell_hook_installed: false,
                        }
                    }
                    AutoSaveChoice::ShareToCommunity => {
                        println!(" {} Auto-save enabled + Community Brain sharing", "+".green());
                        AethrConfig {
                            auto_save: true,
                            share_to_community: true,
                            shell_hook_installed: false,
                        }
                    }
                    AutoSaveChoice::Disabled => {
                        println!(" {} Auto-save disabled", "-".yellow());
                        AethrConfig {
                            auto_save: false,
                            share_to_community: false,
                            shell_hook_installed: false,
                        }
                    }
                };
                
                config.save()?;
            }
            Err(e) => {
                eprintln!(" {} Failed to show consent: {}", "x".red(), e);
            }
        }
    }

    // Show shell hook setup instructions if auto-save is enabled
    let config = AethrConfig::load();
    if config.auto_save && !config.shell_hook_installed {
        println!();
        println!(" {} To enable automatic command saving:", "!".yellow());
        println!("   Run {} to install the shell hook", "aethr hook --install".cyan());
        println!("   Or add to your shell config:");
        println!("     {} (bash)", "eval \"$(aethr hook bash)\"".dimmed());
        println!("     {} (zsh)", "eval \"$(aethr hook zsh)\"".dimmed());
    }

    println!();
    println!(" Aethr initialized. Run {} to import your shell history.", "aethr import".cyan());
    println!();

    Ok(())
}
