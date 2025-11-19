use crate::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct MoatEntry {
    pub id: Option<i64>,
    pub command: String,
    pub context_tags: Option<String>, // JSON array as text
    pub success_score: Option<f32>,
    pub provenance: Option<String>,
    pub created_at: Option<i64>,
}

pub struct CommunityMoat {
    conn: Connection,
}

impl CommunityMoat {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        // Create table and FTS if missing
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS community_moat (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                context_tags TEXT,
                success_score REAL DEFAULT 0.0,
                provenance TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS community_moat_fts USING fts5(
                command,
                context_tags,
                content='community_moat',
                content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS community_moat_ai AFTER INSERT ON community_moat BEGIN
                INSERT INTO community_moat_fts(rowid, command, context_tags)
                VALUES (new.id, new.command, new.context_tags);
            END;

            CREATE TRIGGER IF NOT EXISTS community_moat_ad AFTER DELETE ON community_moat BEGIN
                DELETE FROM community_moat_fts WHERE rowid = old.id;
            END;
            "#,
        )?;
        Ok(Self { conn })
    }

    pub fn insert(&self, entry: MoatEntry) -> Result<()> {
        let ts = entry.created_at.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        });
        self.conn.execute(
            "INSERT INTO community_moat (command, context_tags, success_score, provenance, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.command,
                entry.context_tags,
                entry.success_score,
                entry.provenance,
                ts
            ],
        )?;
        Ok(())
    }

    /// Search moat by simple FTS expression; optional context tag filtering (comma-separated tags)
    pub fn search(&self, query: &str, _context_filter: Option<&[&str]>, limit: usize) -> Result<Vec<String>> {
        // Simplified search ignoring context filter for now (avoids lifetime complexity)
        let tokens: Vec<String> = query
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.replace('"', ""))
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        let expr = tokens.join(" OR ");
        let sql = "SELECT community_moat.command FROM community_moat JOIN community_moat_fts ON community_moat.id = community_moat_fts.rowid WHERE community_moat_fts MATCH ?1 ORDER BY community_moat.success_score DESC, community_moat.created_at DESC LIMIT ?2;";
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![expr, limit as i64], |r| r.get(0))?;
        let mut out = Vec::new();
        for row in rows { out.push(row?); }
        Ok(out)
    }
}