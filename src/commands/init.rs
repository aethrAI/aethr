use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config::{self, AethrConfig};
use crate::ui::{show_consent, AutoSaveChoice};
use std::io::{self, Write};

pub fn run() -> Result<()> {
    let config_dir = config::config_dir();
    let config_file = config_dir.join("config.json");
    
    // Check if this is first run (no config file)
    let is_first_run = !config_file.exists();
    
    // Create config directory silently
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    // Initialize database silently
    let db_path = config::get_db_path();
    let _db = LocalDB::new(&db_path)?;

    // Create token file
    let token_path = config::get_token_path();
    if !token_path.exists() {
        std::fs::write(&token_path, "")?;
    }

    // On first run, show consent prompt
    if is_first_run {
        match show_consent() {
            Ok(choice) => {
                // Clear and show results
                print!("\x1B[2J\x1B[H");
                
                let version = env!("CARGO_PKG_VERSION");
                println!("\x1B[33mWelcome to Aethr\x1B[0m");
                println!("Version {}", version);
                println!();
                
                let config = match choice {
                    AutoSaveChoice::LocalOnly => {
                        println!("\x1B[33m●\x1B[0m Auto-save enabled (local only)");
                        AethrConfig {
                            auto_save: true,
                            share_to_community: false,
                            shell_hook_installed: false,
                        }
                    }
                    AutoSaveChoice::ShareToCommunity => {
                        println!("\x1B[33m●\x1B[0m Auto-save enabled + community sharing");
                        AethrConfig {
                            auto_save: true,
                            share_to_community: true,
                            shell_hook_installed: false,
                        }
                    }
                    AutoSaveChoice::Disabled => {
                        println!("\x1B[90m○\x1B[0m Auto-save disabled");
                        AethrConfig {
                            auto_save: false,
                            share_to_community: false,
                            shell_hook_installed: false,
                        }
                    }
                };
                
                println!("\x1B[33m●\x1B[0m Database initialized");
                println!("\x1B[33m●\x1B[0m Configuration saved");
                
                config.save()?;
                
                // Show next steps
                println!();
                if config.auto_save {
                    println!("\x1B[1mNext steps:\x1B[0m");
                    println!();
                    println!("  1. Install shell hook:");
                    println!("     \x1B[90m$ aethr hook --install\x1B[0m");
                    println!();
                    println!("  2. Import existing history:");
                    println!("     \x1B[90m$ aethr import\x1B[0m");
                } else {
                    println!("\x1B[1mNext step:\x1B[0m");
                    println!();
                    println!("  Import your existing history:");
                    println!("  \x1B[90m$ aethr import\x1B[0m");
                }
                
                println!();
                println!("\x1B[32mInitialization complete!\x1B[0m");
            }
            Err(e) => {
                eprintln!("\x1B[31m✗\x1B[0m Failed: {}", e);
            }
        }
    } else {
        println!("Already initialized. Run \x1B[90maethr status\x1B[0m to see current config.");
    }
    
    io::stdout().flush()?;
    Ok(())
}
