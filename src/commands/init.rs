use crate::Result;
use crate::db::local::LocalDB;
use crate::utils::config::{self, AethrConfig};
use crate::ui::{show_consent, AutoSaveChoice};
use std::io::{self, Write};

fn print_logo() {
    println!();
    println!("\x1B[1;37m        █████╗ ███████╗████████╗██╗  ██╗██████╗ \x1B[0m");
    println!("\x1B[1;37m       ██╔══██╗██╔════╝╚══██╔══╝██║  ██║██╔══██╗\x1B[0m");
    println!("\x1B[1;37m       ███████║█████╗     ██║   ███████║██████╔╝\x1B[0m");
    println!("\x1B[1;37m       ██╔══██║██╔══╝     ██║   ██╔══██║██╔══██╗\x1B[0m");
    println!("\x1B[1;37m       ██║  ██║███████╗   ██║   ██║  ██║██║  ██║\x1B[0m");
    println!("\x1B[1;37m       ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝\x1B[0m");
    println!();
    println!("\x1B[2m       Terminal Intelligence That Learns\x1B[0m");
    println!();
}

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
                // Clear screen and show logo
                print!("\x1B[2J\x1B[H");
                print_logo();
                
                let config = match choice {
                    AutoSaveChoice::LocalOnly => {
                        println!("  \x1B[32m✓\x1B[0m Auto-save enabled \x1B[90m(local only)\x1B[0m");
                        AethrConfig {
                            auto_save: true,
                            share_to_community: false,
                            shell_hook_installed: false,
                        }
                    }
                    AutoSaveChoice::ShareToCommunity => {
                        println!("  \x1B[32m✓\x1B[0m Auto-save enabled \x1B[90m+ Community sharing\x1B[0m");
                        AethrConfig {
                            auto_save: true,
                            share_to_community: true,
                            shell_hook_installed: false,
                        }
                    }
                    AutoSaveChoice::Disabled => {
                        println!("  \x1B[33m○\x1B[0m Auto-save disabled");
                        AethrConfig {
                            auto_save: false,
                            share_to_community: false,
                            shell_hook_installed: false,
                        }
                    }
                };
                
                println!("  \x1B[32m✓\x1B[0m Database initialized");
                
                config.save()?;
            }
            Err(e) => {
                eprintln!("  \x1B[31m✗\x1B[0m Failed to show consent: {}", e);
            }
        }
    } else {
        // Not first run, just show status
        print!("\x1B[2J\x1B[H");
        print_logo();
        println!("  \x1B[32m✓\x1B[0m Already initialized");
    }

    // Show next steps
    let config = AethrConfig::load();
    
    println!();
    println!("  \x1B[1mNext steps:\x1B[0m");
    println!();
    
    if config.auto_save && !config.shell_hook_installed {
        println!("  \x1B[36m1.\x1B[0m Install shell hook for auto-save:");
        println!("     \x1B[90m$ aethr hook --install\x1B[0m");
        println!();
        println!("  \x1B[36m2.\x1B[0m Import your existing history:");
        println!("     \x1B[90m$ aethr import\x1B[0m");
    } else {
        println!("  \x1B[36m●\x1B[0m Import your existing history:");
        println!("     \x1B[90m$ aethr import\x1B[0m");
    }
    
    println!();
    println!("  \x1B[1;32mInitialization complete!\x1B[0m");
    println!();
    
    io::stdout().flush()?;
    
    Ok(())
}
