PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS phases (
    id BLOB PRIMARY KEY,
    project_id BLOB NOT NULL,
    name TEXT NOT NULL CHECK (name IN ('prompt','fix','hardening')),
    created_at TEXT NOT NULL DEFAULT (datetime('now','subsec')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
