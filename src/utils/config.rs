use std::path::{PathBuf};

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
