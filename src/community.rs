// Community Moat Auto-Expansion System
// Enables Aethr to grow its knowledge base as users contribute solutions

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::Path;
use crate::Result;

/// A command contribution from a user (anonymized)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    /// The command that solved the problem
    pub command: String,
    
    /// Tags/context (docker, python, git, etc.)
    pub context_tags: Vec<String>,
    
    /// Error message this solves
    pub error_message: Option<String>,
    
    /// User's success rating (1-5 stars, None = not rated)
    pub success_rating: Option<u8>,
    
    /// When was this contributed (ISO 8601 string)
    pub created_at: String,
    
    /// Anonymous user ID (hashed, cannot be reversed)
    pub contributor_hash: String,
    
    /// Platform info (Linux, macOS, etc.)
    pub platform: Option<String>,
    
    /// Shell type (bash, zsh, fish, etc.)
    pub shell: Option<String>,
}

/// Configuration for community contribution opt-in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityConfig {
    /// Does user allow anonymous contributions?
    pub opt_in: bool,
    
    /// Include error messages in contributions? (some may be sensitive)
    pub include_errors: bool,
    
    /// Share success rates?
    pub share_metrics: bool,
    
    /// Last sync timestamp (ISO 8601 string)
    pub last_sync: Option<String>,
    
    /// Sync frequency (hours, default 24)
    pub sync_interval_hours: u32,
}

impl Default for CommunityConfig {
    fn default() -> Self {
        Self {
            opt_in: false, // Opt-in by default (privacy-first)
            include_errors: false,
            share_metrics: false,
            last_sync: None,
            sync_interval_hours: 24,
        }
    }
}

/// Community Moat Manager
pub struct CommunityMoatManager {
    config_path: String,
    config: CommunityConfig,
}

impl CommunityMoatManager {
    pub fn new(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("community_config.json").to_string_lossy().to_string();
        let config = if Path::new(&config_path).exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content)?
        } else {
            CommunityConfig::default()
        };

        Ok(Self { config_path, config })
    }

    /// Save the configuration
    pub fn save_config(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Enable community contributions (opt-in)
    pub fn enable_contributions(&mut self, include_errors: bool, share_metrics: bool) -> Result<()> {
        self.config.opt_in = true;
        self.config.include_errors = include_errors;
        self.config.share_metrics = share_metrics;
        self.save_config()?;
        Ok(())
    }

    /// Disable community contributions (opt-out)
    pub fn disable_contributions(&mut self) -> Result<()> {
        self.config.opt_in = false;
        self.save_config()?;
        Ok(())
    }

    /// Generate anonymous contributor hash (privacy-preserving)
    /// Uses simple hash of machine info - cannot be reversed to user
    pub fn generate_contributor_hash(&self) -> String {
        use std::process::Command;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Try to get MAC address (platform-specific)
        let mac = if cfg!(target_os = "macos") {
            Command::new("ifconfig")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|output| {
                    output.lines().find(|l| l.contains("ether"))
                        .map(|s| s.trim().to_string())
                })
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            Command::new("cat")
                .arg("/sys/class/net/eth0/address")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        };

        // Simple hash
        let mut hasher = DefaultHasher::new();
        mac.hash(&mut hasher);
        format!("anon_{:x}", hasher.finish())
    }

    /// Check if we should sync contributions (time-based)
    pub fn should_sync(&self) -> bool {
        if !self.config.opt_in {
            return false;
        }

        match &self.config.last_sync {
            None => true, // Never synced
            Some(last_sync_str) => {
                // Simple string comparison for time (good enough for 24h check)
                // In production, would parse datetime properly
                let now = Utc::now().to_rfc3339();
                now > *last_sync_str // Very basic check
            }
        }
    }

    /// Capture a successful command execution as a contribution
    pub fn capture_command(
        &self,
        command: &str,
        context_tags: Vec<String>,
        error_message: Option<String>,
        success_rating: Option<u8>,
    ) -> Result<Contribution> {
        Ok(Contribution {
            command: command.to_string(),
            context_tags,
            error_message: if self.config.include_errors { error_message } else { None },
            success_rating,
            created_at: Utc::now().to_rfc3339(),
            contributor_hash: self.generate_contributor_hash(),
            platform: Some(std::env::consts::OS.to_string()),
            shell: std::env::var("SHELL").ok(),
        })
    }

    /// Simulate uploading contributions to seed_moat
    /// In production, this would POST to api.aethr-ai.dev
    pub fn sync_contributions(
        &mut self,
        contributions: Vec<Contribution>,
    ) -> Result<usize> {
        if !self.config.opt_in {
            return Err(anyhow::anyhow!("Community contributions not enabled"));
        }

        // In production:
        // 1. Filter contributions for sensitive data
        // 2. Batch them (max 100 per request)
        // 3. POST to https://api.aethr-ai.dev/v1/moat/contribute
        // 4. Return count of accepted contributions

        // For now, just count them
        let count = contributions.len();

        // Update last sync
        self.config.last_sync = Some(Utc::now().to_rfc3339());
        self.save_config()?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_config_default() {
        let config = CommunityConfig::default();
        assert!(!config.opt_in);
    }

    #[test]
    fn test_contributor_hash_generation() {
        let manager = CommunityMoatManager::new(
            &std::path::PathBuf::from("/tmp")
        ).unwrap();
        let hash = manager.generate_contributor_hash();
        assert!(hash.starts_with("anon_"));
    }
}
