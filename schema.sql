-- Command history table
CREATE TABLE IF NOT EXISTS command_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    working_dir TEXT NOT NULL,
    context_tags TEXT,  -- JSON array
    exit_code INTEGER,
    duration_ms INTEGER,
    timestamp INTEGER NOT NULL,
    command_normalized TEXT,
    UNIQUE(command, working_dir, timestamp)
);

-- Full-text search FTS5 virtual table
CREATE VIRTUAL TABLE IF NOT EXISTS command_fts USING fts5(
    command,
    command_normalized,
    context_tags,
    content='command_history',
    content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS command_history_ai AFTER INSERT ON command_history BEGIN
    INSERT INTO command_fts(rowid, command, command_normalized, context_tags)
    VALUES (new.id, new.command, new.command_normalized, new.context_tags);
END;

CREATE TRIGGER IF NOT EXISTS command_history_ad AFTER DELETE ON command_history BEGIN
    DELETE FROM command_fts WHERE rowid = old.id;
END;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_timestamp ON command_history(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_context ON command_history(context_tags);
CREATE INDEX IF NOT EXISTS idx_exit_code ON command_history(exit_code);

-- Config table
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Project contexts
CREATE TABLE IF NOT EXISTS project_contexts (
    path TEXT PRIMARY KEY,
    context_tags TEXT NOT NULL,
    last_detected INTEGER NOT NULL
);
