use crate::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainEntry {
    pub id: Option<i64>,
    pub command: String,
    pub error_pattern: Option<String>,
    pub context_tags: Option<String>,
    pub success_count: i64,
    pub fail_count: i64,
    pub provenance: Option<String>,
    pub created_at: Option<i64>,
}

impl BrainEntry {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.fail_count;
        if total == 0 {
            return 0.0;
        }
        (self.success_count as f32 / total as f32) * 100.0
    }
}

/// Result from Community Brain search with scoring
#[derive(Debug, Clone)]
pub struct BrainResult {
    pub command: String,
    pub error_pattern: Option<String>,
    pub context_tags: Option<String>,
    pub success_count: i64,
    pub fail_count: i64,
    pub success_rate: f32,
    pub score: f32,
}

pub struct CommunityBrain {
    conn: Connection,
}

impl CommunityBrain {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS community_brain (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                error_pattern TEXT,
                context_tags TEXT,
                success_count INTEGER DEFAULT 0,
                fail_count INTEGER DEFAULT 0,
                provenance TEXT,
                created_at INTEGER NOT NULL,
                UNIQUE(command, error_pattern)
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS community_brain_fts USING fts5(
                command,
                error_pattern,
                context_tags,
                content='community_brain',
                content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS community_brain_ai AFTER INSERT ON community_brain BEGIN
                INSERT INTO community_brain_fts(rowid, command, error_pattern, context_tags)
                VALUES (new.id, new.command, new.error_pattern, new.context_tags);
            END;

            CREATE TRIGGER IF NOT EXISTS community_brain_ad AFTER DELETE ON community_brain BEGIN
                DELETE FROM community_brain_fts WHERE rowid = old.id;
            END;
            "#,
        )?;
        Ok(Self { conn })
    }

    /// Insert a new fix or update existing one
    pub fn insert(&self, entry: BrainEntry) -> Result<()> {
        let ts = entry.created_at.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        });
        
        // Try to insert, on conflict update counts
        self.conn.execute(
            "INSERT INTO community_brain (command, error_pattern, context_tags, success_count, fail_count, provenance, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(command, error_pattern) DO UPDATE SET
                success_count = success_count + excluded.success_count,
                fail_count = fail_count + excluded.fail_count",
            params![
                entry.command,
                entry.error_pattern,
                entry.context_tags,
                entry.success_count,
                entry.fail_count,
                entry.provenance,
                ts
            ],
        )?;
        Ok(())
    }

    /// Log a successful fix (increments success count)
    pub fn log_success(&self, command: &str, error_pattern: &str, context_tags: Option<&str>) -> Result<()> {
        let entry = BrainEntry {
            id: None,
            command: command.to_string(),
            error_pattern: Some(error_pattern.to_string()),
            context_tags: context_tags.map(|s| s.to_string()),
            success_count: 1,
            fail_count: 0,
            provenance: Some("user".to_string()),
            created_at: None,
        };
        self.insert(entry)
    }

    /// Log a failed fix attempt (increments fail count)
    pub fn log_failure(&self, command: &str, error_pattern: &str) -> Result<()> {
        // Only update if exists
        self.conn.execute(
            "UPDATE community_brain SET fail_count = fail_count + 1 WHERE command = ?1 AND error_pattern = ?2",
            params![command, error_pattern],
        )?;
        Ok(())
    }

    /// Search for fixes with full scoring
    pub fn search_with_scores(&self, query: &str, context_tags: Option<&[&str]>, limit: usize) -> Result<Vec<BrainResult>> {
        let tokens: Vec<String> = query
            .split_whitespace()
            .filter(|s| !s.is_empty() && s.len() > 2)
            .filter(|s| s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
            .map(|s| format!("\"{}\"", s)) // Quote each token for FTS5
            .collect();
        
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        
        let expr = tokens.join(" OR ");
        
        let sql = "
            SELECT 
                community_brain.command,
                community_brain.error_pattern,
                community_brain.context_tags,
                community_brain.success_count,
                community_brain.fail_count
            FROM community_brain 
            JOIN community_brain_fts ON community_brain.id = community_brain_fts.rowid 
            WHERE community_brain_fts MATCH ?1 
            ORDER BY community_brain.success_count DESC, community_brain.created_at DESC 
            LIMIT ?2;
        ";
        
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![expr, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            let (command, error_pattern, ctx_tags, success_count, fail_count) = row?;
            
            let total = success_count + fail_count;
            let success_rate = if total > 0 {
                (success_count as f32 / total as f32) * 100.0
            } else {
                50.0 // New entries get 50% default
            };
            
            // Score based on success rate and total uses
            let mut score = success_rate;
            
            // Boost for context match
            if let (Some(ref entry_tags), Some(filter_tags)) = (&ctx_tags, context_tags) {
                for tag in filter_tags {
                    if entry_tags.contains(tag) {
                        score *= 1.5;
                    }
                }
            }
            
            // Boost for high usage
            if total > 10 {
                score *= 1.2;
            }
            
            results.push(BrainResult {
                command,
                error_pattern,
                context_tags: ctx_tags,
                success_count,
                fail_count,
                success_rate,
                score,
            });
        }
        
        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }

    /// Simple search (backwards compatible)
    pub fn search(&self, query: &str, context_filter: Option<&[&str]>, limit: usize) -> Result<Vec<String>> {
        let results = self.search_with_scores(query, context_filter, limit)?;
        Ok(results.into_iter().map(|r| r.command).collect())
    }
    
    /// Get entry count
    pub fn count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM community_brain",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Seed the Community Brain with common error fixes
    /// This provides a starting knowledge base for new users
    pub fn seed_if_empty(&self) -> Result<()> {
        let count = self.count()?;
        if count > 0 {
            return Ok(()); // Already has data
        }

        // Common error fixes from the developer community
        let seed_data = vec![
            // Node.js / npm errors
            ("npm install", "npm ERR! code ENOENT", "nodejs,javascript", 50, 5),
            ("npm install", "module not found", "nodejs,javascript", 45, 4),
            ("npm cache clean --force && npm install", "npm ERR! code EINTEGRITY", "nodejs,javascript", 38, 3),
            ("rm -rf node_modules && npm install", "npm ERR! peer dep missing", "nodejs,javascript", 42, 6),
            ("npm audit fix", "npm WARN deprecated", "nodejs,javascript", 30, 2),
            ("npm install --legacy-peer-deps", "ERESOLVE unable to resolve dependency tree", "nodejs,javascript", 55, 8),
            ("npx create-react-app . --template typescript", "create-react-app command not found", "nodejs,react", 25, 2),
            
            // Python errors
            ("pip install -r requirements.txt", "ModuleNotFoundError: No module named", "python", 60, 5),
            ("pip install numpy", "no module named numpy", "python", 35, 3),
            ("python -m venv venv && source venv/bin/activate", "externally-managed-environment", "python", 28, 2),
            ("pip install --upgrade pip", "pip is configured with locations that require TLS", "python", 22, 1),
            ("pip3 install", "command not found: pip", "python", 40, 4),
            
            // Git errors
            ("git config --global http.sslVerify false", "SSL certificate problem", "git", 30, 5),
            ("git pull --rebase origin main", "Your local changes would be overwritten", "git", 35, 4),
            ("git stash && git pull && git stash pop", "Please commit your changes or stash them", "git", 45, 3),
            ("git remote set-url origin git@github.com:user/repo.git", "could not resolve host: github.com", "git", 25, 2),
            ("git checkout -b main && git branch -D master", "fatal: 'master' is not a git repository", "git", 20, 2),
            ("git config --global init.defaultBranch main", "warning: the 'master' branch", "git", 18, 1),
            
            // Rust / Cargo errors
            ("cargo clean && cargo build", "could not compile", "rust", 40, 5),
            ("cargo update", "failed to select a version", "rust", 35, 3),
            ("rustup update", "error: linker 'cc' not found", "rust", 28, 2),
            ("cargo build --release", "memory allocation failed", "rust", 22, 4),
            
            // Docker errors
            ("docker system prune -a", "no space left on device", "docker", 50, 4),
            ("docker-compose down && docker-compose up -d", "port is already allocated", "docker", 45, 5),
            ("sudo systemctl start docker", "Cannot connect to the Docker daemon", "docker", 55, 3),
            ("docker pull", "TLS handshake timeout", "docker", 25, 3),
            
            // Permission errors (Linux/macOS)
            ("sudo chown -R $USER:$USER .", "EACCES: permission denied", "linux,macos", 60, 8),
            ("chmod +x", "Permission denied", "linux,macos", 55, 5),
            ("sudo", "Operation not permitted", "linux,macos", 50, 6),
            
            // Database errors
            ("sudo systemctl start postgresql", "connection refused", "postgresql", 35, 3),
            ("mysql -u root -p", "Access denied for user 'root'", "mysql", 30, 4),
            ("redis-server", "Address already in use", "redis", 28, 2),
            
            // Build tool errors
            ("make clean && make", "make: *** No rule to make target", "c,cpp", 32, 4),
            ("cmake -B build && cmake --build build", "CMake Error", "cmake", 28, 3),
            
            // Network/CORS errors
            ("curl -k", "SSL certificate verify failed", "api,http", 35, 4),
            
            // TypeScript errors
            ("npx tsc --init", "Cannot find module 'typescript'", "typescript", 40, 3),
            ("npm install @types/node --save-dev", "Could not find a declaration file", "typescript", 45, 4),
        ];

        for (command, error, tags, success, fail) in seed_data {
            let entry = BrainEntry {
                id: None,
                command: command.to_string(),
                error_pattern: Some(error.to_string()),
                context_tags: Some(tags.to_string()),
                success_count: success,
                fail_count: fail,
                provenance: Some("seed".to_string()),
                created_at: Some(1700000000), // Fixed timestamp for seed data
            };
            let _ = self.insert(entry); // Ignore errors for duplicate entries
        }

        Ok(())
    }
}
