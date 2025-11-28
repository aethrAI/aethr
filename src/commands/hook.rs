use crate::Result;
use crate::utils::config::{self, AethrConfig};
use colored::*;
use std::fs;
use std::path::PathBuf;

/// Get shell-specific eval command
fn get_eval_command(shell: &str) -> String {
    match shell {
        "bash" => format!(r#"
# Aethr auto-save hook
_aethr_preexec() {{
    local cmd="$BASH_COMMAND"
    [[ -z "$cmd" ]] && return
    [[ "$cmd" == aethr* ]] && return
    [[ "$cmd" == "$PROMPT_COMMAND" ]] && return
    [[ -n "$COMP_LINE" ]] && return
    local timestamp=$(date +%s)
    echo "${{timestamp}}|$PWD|${{cmd}}" >> ~/.aethr/commands.log
}}
trap '_aethr_preexec' DEBUG
"#),
        "zsh" => format!(r#"
# Aethr auto-save hook
_aethr_preexec() {{
    local cmd="$1"
    [[ -z "$cmd" ]] && return
    [[ "$cmd" == aethr* ]] && return
    local timestamp=$(date +%s)
    echo "${{timestamp}}|$PWD|${{cmd}}" >> ~/.aethr/commands.log
}}
autoload -Uz add-zsh-hook
add-zsh-hook preexec _aethr_preexec
"#),
        _ => String::new(),
    }
}

pub fn run(shell: Option<&str>) -> Result<()> {
    let config = AethrConfig::load();
    
    // If auto_save is disabled, don't output hook
    if !config.auto_save {
        eprintln!("{}", "Auto-save is disabled. Run 'aethr init' to enable.".yellow());
        return Ok(());
    }
    
    match shell {
        Some("bash") => {
            print!("{}", get_eval_command("bash"));
        }
        Some("zsh") => {
            print!("{}", get_eval_command("zsh"));
        }
        Some(s) => {
            eprintln!("{} Unsupported shell: {}", "x".red(), s);
            eprintln!("  Supported shells: bash, zsh");
        }
        None => {
            // Show setup instructions
            println!();
            println!("{}", "Shell Hook Setup".bold());
            println!();
            println!("Add one of these lines to your shell config:");
            println!();
            println!("  {} (in ~/.bashrc)", "eval \"$(aethr hook bash)\"".cyan());
            println!("  {} (in ~/.zshrc)", "eval \"$(aethr hook zsh)\"".cyan());
            println!();
            println!("Then reload your shell:");
            println!("  {} or {}", "source ~/.bashrc".cyan(), "source ~/.zshrc".cyan());
            println!();
        }
    }
    
    Ok(())
}

/// Process logged commands and import to database
pub fn process_log() -> Result<usize> {
    let log_path = config::get_cmd_log_path();
    
    if !log_path.exists() {
        return Ok(0);
    }
    
    let content = fs::read_to_string(&log_path)?;
    if content.trim().is_empty() {
        return Ok(0);
    }
    
    let db_path = config::get_db_path();
    let db = crate::db::local::LocalDB::new(&db_path)?;
    
    let mut count = 0;
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 {
            let timestamp: i64 = parts[0].parse().unwrap_or(0);
            let dir = parts[1];
            let cmd = parts[2].trim();
            
            if !cmd.is_empty() && timestamp > 0 {
                // Insert with exit_code 0 (we don't track exit codes in hook)
                if db.insert_command(cmd, dir, 0, timestamp).is_ok() {
                    count += 1;
                }
            }
        }
    }
    
    // Clear the log after processing
    fs::write(&log_path, "")?;
    
    Ok(count)
}

/// Setup shell hook (write instructions to shell config)
pub fn setup() -> Result<()> {
    let config = AethrConfig::load();
    
    if !config.auto_save {
        println!();
        println!("{} Auto-save is disabled.", "!".yellow());
        println!("  Run {} to enable auto-save first.", "aethr init".cyan());
        println!();
        return Ok(());
    }
    
    // Detect shell
    let shell = std::env::var("SHELL").unwrap_or_default();
    let (rc_file, shell_name): (PathBuf, &str) = if shell.contains("zsh") {
        (dirs::home_dir().unwrap().join(".zshrc"), "zsh")
    } else {
        (dirs::home_dir().unwrap().join(".bashrc"), "bash")
    };
    
    // Check if already installed
    if rc_file.exists() {
        let content = fs::read_to_string(&rc_file)?;
        if content.contains("aethr hook") {
            println!();
            println!("{} Shell hook already installed in {}", "+".green(), rc_file.display());
            println!();
            return Ok(());
        }
    }
    
    // Add hook to shell config
    let hook_line = format!("\n# Aethr auto-save\neval \"$(aethr hook {})\"\n", shell_name);
    
    let mut content = if rc_file.exists() {
        fs::read_to_string(&rc_file)?
    } else {
        String::new()
    };
    
    content.push_str(&hook_line);
    fs::write(&rc_file, content)?;
    
    // Update config
    let mut config = config;
    config.shell_hook_installed = true;
    config.save()?;
    
    println!();
    println!("{} Shell hook installed in {}", "+".green(), rc_file.display());
    println!();
    println!("Reload your shell to activate:");
    println!("  {}", format!("source {}", rc_file.display()).cyan());
    println!();
    
    Ok(())
}
