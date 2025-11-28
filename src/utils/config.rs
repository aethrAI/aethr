use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// User configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AethrConfig {
    #[serde(default)]
    pub auto_save: bool,
    #[serde(default)]
    pub share_to_community: bool,
    #[serde(default)]
    pub shell_hook_installed: bool,
}

impl AethrConfig {
    /// Load config from file, or return defaults
    pub fn load() -> Self {
        let config_path = config_dir().join("config.json");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save config to file
    pub fn save(&self) -> std::io::Result<()> {
        let config_path = config_dir().join("config.json");
        std::fs::create_dir_all(config_dir())?;
        std::fs::write(&config_path, serde_json::to_string_pretty(self)?)
    }
}

/// Returns the root configuration directory (~/.aethr)
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aethr")
}

/// Path to the local SQLite database
pub fn get_db_path() -> PathBuf {
    config_dir().join("aethr.db")
}

/// Path to the command log produced by shell hook
pub fn get_cmd_log_path() -> PathBuf {
    config_dir().join("commands.log")
}

/// Path to the shell hook script
pub fn get_hook_script_path() -> PathBuf {
    config_dir().join("hook.sh")
}

/// Unix socket path used by daemon
pub fn get_daemon_socket_path() -> PathBuf {
    config_dir().join("daemon.sock")
}

/// Stored API token path (simple text file for now)
pub fn get_token_path() -> PathBuf {
    config_dir().join("token")
}

/// Load API token if present
pub fn load_token() -> Option<String> {
    let path = get_token_path();
    std::fs::read_to_string(&path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Persist API token
pub fn save_token(token: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    std::fs::write(get_token_path(), format!("{}\n", token.trim()))
}
