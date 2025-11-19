use crate::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata about a command for scoring and ranking
#[derive(Debug, Clone)]
pub struct CommandScore {
    pub command: String,
    pub timestamp: i64,
    pub frequency: i64,
    pub recency_score: f32,
    pub frequency_score: f32,
    pub combined_score: f32,
}

/// LocalDB wraps SQLite operations (FTS5)
pub struct LocalDB {
    conn: Connection,
}

impl LocalDB {
    /// Open or create DB and apply schema.sql
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        let schema = include_str!("../../schema.sql");
        conn.execute_batch(schema)?;
        Ok(Self { conn })
    }

    pub fn insert_command(
        &self,
        command: &str,
        working_dir: &str,
        exit_code: i32,
        timestamp: i64,
    ) -> Result<()> {
        let command_normalized = Self::normalize(command);
        self.conn.execute(
            "INSERT OR IGNORE INTO command_history (command, working_dir, context_tags, exit_code, duration_ms, timestamp, command_normalized)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                command,
                working_dir,
                Option::<String>::None,
                exit_code,
                Option::<i64>::None,
                timestamp,
                command_normalized
            ],
        )?;
        Ok(())
    }

    fn normalize(cmd: &str) -> String {
        cmd.trim().to_lowercase()
    }

    /// Search FTS5 for query tokens. Returns raw command strings.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let tokens: Vec<String> = query
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.replace('"', ""))
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        let expr = tokens.join(" OR ");
        let sql = "
            SELECT command_history.command
            FROM command_history
            JOIN command_fts ON command_history.id = command_fts.rowid
            WHERE command_fts MATCH ?1
            ORDER BY timestamp DESC
            LIMIT ?2;
        ";
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![expr, limit as i64], |r| r.get(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Search with scoring: returns commands with recency + frequency scores
    pub fn search_with_scores(&self, query: &str, limit: usize) -> Result<Vec<CommandScore>> {
        let tokens: Vec<String> = query
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.replace('"', ""))
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        let expr = tokens.join(" OR ");

        // Get current timestamp for recency calculation
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 24 hours in seconds
        let twenty_four_hours = 24 * 60 * 60;

        let sql = "
            SELECT 
                command_history.command,
                command_history.timestamp,
                COUNT(*) as frequency
            FROM command_history
            JOIN command_fts ON command_history.id = command_fts.rowid
            WHERE command_fts MATCH ?1
            GROUP BY command_history.command_normalized
            ORDER BY timestamp DESC
            LIMIT ?2;
        ";
        
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![expr, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            let (command, timestamp, frequency) = row_result?;

            // Calculate recency score (0.0 to 1.0, higher = more recent)
            let age_seconds = (now - timestamp).max(0);
            let recency_score = if age_seconds < twenty_four_hours {
                1.0 - (age_seconds as f32 / twenty_four_hours as f32) * 0.5
            } else {
                0.5 - ((age_seconds - twenty_four_hours) as f32 / (30 * 24 * 60 * 60) as f32) * 0.4
            }
            .max(0.1);

            // Calculate frequency score (0.0 to 1.0, higher = more frequent)
            let frequency_score = (frequency.min(100) as f32 / 100.0).min(1.0).max(0.1);

            // Combined score: 60% recency + 40% frequency
            let combined_score = (recency_score * 0.6) + (frequency_score * 0.4);

            results.push(CommandScore {
                command,
                timestamp,
                frequency,
                recency_score,
                frequency_score,
                combined_score,
            });
        }

        // Sort by combined score (highest first)
        results.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }
}
