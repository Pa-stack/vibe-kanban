PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS task_context_files (
    id BLOB PRIMARY KEY,
    task_id BLOB NOT NULL,
    filename TEXT NOT NULL,
    mime TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    stored_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now','subsec')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_task_context_files_task ON task_context_files(task_id);
