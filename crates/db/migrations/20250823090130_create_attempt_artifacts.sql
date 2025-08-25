PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS attempt_artifacts (
    id BLOB PRIMARY KEY,
    attempt_id BLOB NOT NULL,
    kind TEXT NOT NULL,
    path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now','subsec')),
    FOREIGN KEY (attempt_id) REFERENCES task_attempts(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_attempt_artifacts_attempt ON attempt_artifacts(attempt_id);
