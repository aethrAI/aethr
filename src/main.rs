use clap::{Parser, Subcommand};
use aethr_core::ui::run_interactive;
use aethr_core::utils::config::AethrConfig;

#[derive(Parser)]
#[command(name = "aethr")]
#[command(about = "Terminal Intelligence That Learns")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Aethr (create database and config)
    Init,
    
    /// Import shell history
    Import,
    
    /// Search your command history
    Recall {
        #[arg(help = "Search query", num_args = 1.., value_delimiter = ' ')]
        query: Vec<String>,
    },
    
    /// Fix a terminal error
    Fix {
        #[arg(help = "Error message", num_args = 1.., value_delimiter = ' ')]
        error: Vec<String>,
    },
    
    /// Authenticate with your token
    Login {
        #[arg(help = "Your Aethr token")]
        token: String,
    },
    
    /// Check Aethr status
    Status,
    
    /// Shell hook for auto-saving commands
    Hook {
        #[arg(help = "Shell type (bash or zsh)")]
        shell: Option<String>,
        
        #[arg(long, help = "Install hook to shell config")]
        install: bool,
    },
}

fn main() {
    use colored::*;
    
    // Silently process any auto-saved commands from shell hook
    let config = AethrConfig::load();
    if config.auto_save {
        let _ = aethr_core::commands::hook::process_log();
    }
    
    let cli = Cli::parse();
    
    if cli.command.is_none() {
        match run_interactive() {
            Ok(Some((cmd, args))) => {
                let result = match cmd.as_str() {
                    "recall" => {
                        if args.is_empty() {
                            print!("  {} ", "Search:".dimmed());
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                            let mut query = String::new();
                            std::io::stdin().read_line(&mut query).ok();
                            aethr_core::commands::recall::run(query.trim())
                        } else {
                            aethr_core::commands::recall::run(&args)
                        }
                    }
                    "fix" => {
                        if args.is_empty() {
                            print!("  {} ", "Error:".dimmed());
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                            let mut error = String::new();
                            std::io::stdin().read_line(&mut error).ok();
                            aethr_core::commands::fix::run(error.trim())
                        } else {
                            aethr_core::commands::fix::run(&args)
                        }
                    }
                    "import" => aethr_core::commands::import::run(),
                    "init" => aethr_core::commands::init::run(),
                    "status" => aethr_core::commands::status::run(),
                    "login" => {
                        if args.is_empty() {
                            print!("  {} ", "Token:".dimmed());
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                            let mut token = String::new();
                            std::io::stdin().read_line(&mut token).ok();
                            aethr_core::commands::login::run(token.trim())
                        } else {
                            aethr_core::commands::login::run(&args)
                        }
                    }
                    "help" => {
                        println!();
                        println!("  {}", "Commands:".bold());
                        println!("    {}   Search your command history", "/recall".cyan());
                        println!("    {}       Fix a terminal error", "/fix".cyan());
                        println!("    {}    Import shell history", "/import".cyan());
                        println!("    {}      Initialize Aethr", "/init".cyan());
                        println!("    {}    Check status", "/status".cyan());
                        println!("    {}     Authenticate", "/login".cyan());
                        println!("    {}      Exit", "/exit".cyan());
                        println!();
                        Ok(())
                    }
                    "exit" => {
                        println!("  Goodbye");
                        return;
                    }
                    "query" => {
                        aethr_core::commands::recall::run(&args)
                    }
                    _ => {
                        eprintln!(" {} Unknown command: {}", "x".red(), cmd);
                        Ok(())
                    }
                };
                
                if let Err(e) = result {
                    eprintln!(" {} {}", "x".red(), e);
                    std::process::exit(1);
                }
            }
            Ok(None) => {
                println!("  Goodbye");
            }
            Err(e) => {
                eprintln!(" {} {}", "x".red(), e);
                std::process::exit(1);
            }
        }
        return;
    }

    let result = match cli.command.unwrap() {
        Commands::Init => aethr_core::commands::init::run(),
        Commands::Import => aethr_core::commands::import::run(),
        Commands::Recall { query } => aethr_core::commands::recall::run(&query.join(" ")),
        Commands::Fix { error } => aethr_core::commands::fix::run(&error.join(" ")),
        Commands::Login { token } => aethr_core::commands::login::run(&token),
        Commands::Status => aethr_core::commands::status::run(),
        Commands::Hook { shell, install } => {
            if install {
                aethr_core::commands::hook::setup()
            } else {
                aethr_core::commands::hook::run(shell.as_deref())
            }
        }
    };
    
    if let Err(e) = result {
        eprintln!(" {} {}", "x".red(), e);
        std::process::exit(1);
    }
}
