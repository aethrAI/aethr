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
}
