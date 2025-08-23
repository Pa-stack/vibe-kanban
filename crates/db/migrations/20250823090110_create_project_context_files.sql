PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS project_context_files (
    id BLOB PRIMARY KEY,
    project_id BLOB NOT NULL,
    filename TEXT NOT NULL,
    mime TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    stored_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now','subsec')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_project_context_files_project ON project_context_files(project_id);
