use clap::{Parser, Subcommand};
use aethr_core::ui::CommandMenu;

#[derive(Parser)]
#[command(name = "aethr")]
#[command(about = "Context-Gated Terminal Intelligence for Developers")]
#[command(long_about = "Aethr is your personal terminal assistant. It recalls past commands, predicts the next ones, and fixes errors—using your local machine and the global developer community.

KEY FEATURES:
  • recall   - Semantic search through your command history
  • predict  - Get AI-powered command suggestions  
  • fix      - Instant fixes for terminal errors
  • import   - Import shell history for semantic search

QUICK START:
  1. aethr init              # Initialize local database
  2. aethr import            # Import your shell history
  3. aethr recall \"<query>\"  # Search your commands
  4. aethr predict \"<task>\"  # Get command suggestions
  5. aethr fix \"<error>\"     # Fix a command error

Run 'aethr' without arguments to see the interactive menu.
For more help on any command, use: aethr <command> --help")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI for exploring commands
    #[command(alias = "tui", alias = "ui")]
    Run,
    
    /// Initialize Aethr (create database and config files)
    #[command(long_about = "Creates ~/.aethr/ directory with SQLite database and configuration. \
                           Run this once to get started with Aethr.")]
    Init,
    
    /// Import shell history from ~/.aethr/commands.log
    #[command(long_about = "Imports commands from ~/.aethr/commands.log into the local database. \
                           First, add the shell hook to ~/.bashrc or ~/.zshrc, then run this command. \
                           Commands are indexed for semantic search (recall).")]
    Import,
    
    /// Search local command history semantically
    #[command(long_about = "Search through your command history using semantic search. \
                           Examples: 
                             aethr recall \"docker build image\"
                             aethr recall \"git merge conflict\"
                             aethr recall \"npm install deps\"")]
    Recall {
        #[arg(help = "Search query (natural language)")]
        query: String,
    },
    
    /// Predict the next command based on your intent
    #[command(long_about = "Get AI-powered command suggestions based on natural language. \
                           Requires ANTHROPIC_API_KEY for LLM features. \
                           Examples:
                             aethr predict \"build docker image\"
                             aethr predict \"deploy to kubernetes\"
                             aethr predict \"list large files\"")]
    Predict {
        #[arg(help = "Natural language description of what you want to do")]
        prompt: String,
    },
    
    /// Fix a command error
    #[command(long_about = "Get instant fixes for terminal errors. Uses deterministic rules + community knowledge. \
                           Examples:
                             aethr fix \"permission denied\"
                             aethr fix \"Cannot find module express\"
                             aethr fix \"address already in use\"")]
    Fix {
        #[arg(help = "Error message text")]
        error: String,
    },
    
    /// Authenticate with your Aethr token
    #[command(long_about = "Store your authentication token for cloud features. \
                           Get a token from console.aethr-ai.dev \
                           Usage: aethr login <token>")]
    Login {
        #[arg(help = "Your Aethr authentication token")]
        token: String,
    },
    
    /// Start the background daemon (experimental)
    #[command(long_about = "Starts the Aethr daemon for background sync and LLM features. \
                           The daemon runs in the foreground for now.")]
    DaemonStart,
    
    /// Stop the background daemon
    DaemonStop,
    
    /// Check Aethr status and configuration
    #[command(long_about = "Shows authentication status, database location, and token info.")]
    Status,
    
    /// Seed the local database with community fixes
    #[command(long_about = "Populates the database with 50+ curated community command fixes. \
                           Run this after 'aethr init' to get instant suggestions.")]
    SeedMoat,
}

fn main() {
    use colored::*;
    
    let cli = Cli::parse();
    
    // If no command provided, show interactive menu
    let command_to_run = if cli.command.is_none() {
        let mut menu = CommandMenu::new();
        match menu.run() {
            Ok(Some(cmd)) => {
                match cmd.as_str() {
                    "recall" => Some(Commands::Recall { query: String::new() }),
                    "fix" => Some(Commands::Fix { error: String::new() }),
                    "predict" => Some(Commands::Predict { prompt: String::new() }),
                    "import" => Some(Commands::Import),
                    "init" => Some(Commands::Init),
                    "seed-moat" => Some(Commands::SeedMoat),
                    "status" => Some(Commands::Status),
                    "daemon-start" => Some(Commands::DaemonStart),
                    "daemon-stop" => Some(Commands::DaemonStop),
                    _ => Some(Commands::Run),
                }
            }
            Ok(None) => {
                println!("{}","Goodbye!".cyan());
                return;
            }
            Err(e) => {
                eprintln!("{}", format!("Failed to show menu: {}", e).red());
                return;
            }
        }
    } else {
        cli.command
    };

    let result = match command_to_run {
        None | Some(Commands::Run) => aethr_core::ui::interactive::run(),
        Some(Commands::Init) => aethr_core::commands::init::run(),
        Some(Commands::Import) => aethr_core::commands::import::run(),
        Some(Commands::Recall { query }) => {
            if query.is_empty() {
                eprintln!("{}", "Please provide a search query: aethr recall \"<query>\"".red());
                std::process::exit(1);
            }
            aethr_core::commands::recall::run(&query)
        },
        Some(Commands::Predict { prompt }) => {
            if prompt.is_empty() {
                eprintln!("{}", "Please provide a prompt: aethr predict \"<task>\"".red());
                std::process::exit(1);
            }
            aethr_core::commands::predict::run(&prompt)
        },
        Some(Commands::Fix { error }) => {
            if error.is_empty() {
                eprintln!("{}", "Please provide an error message: aethr fix \"<error>\"".red());
                std::process::exit(1);
            }
            aethr_core::commands::fix::run(&error)
        },
        Some(Commands::Login { token }) => aethr_core::commands::login::run(&token),
        Some(Commands::DaemonStart) => {
            println!("Starting daemon (running in foreground for now).");
            aethr_core::daemon::server::run_blocking()
        }
        Some(Commands::DaemonStop) => {
            println!("{}","Daemon stop not implemented yet. Use Ctrl+C on the daemon process.".yellow());
            Ok(())
        }
        Some(Commands::Status) => aethr_core::commands::status::run(),
        Some(Commands::SeedMoat) => aethr_core::commands::seed_moat::run(),
    };
    
    if let Err(e) = result {
        eprintln!("{}", format!("❌ Error: {}", e).red());
        std::process::exit(1);
    }
}