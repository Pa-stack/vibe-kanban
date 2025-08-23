PRAGMA foreign_keys = ON;
-- NOTE: This migration is additive and intentionally lacks a DOWN section.
-- The project's migration style uses single-file forward migrations.
-- All changes here create new tables/columns only; reversing on SQLite would be destructive.
-- Orchestrator M1 scaffold: phases, uploads, artifacts, telemetry fields (minimal placeholders)

-- Phases table
CREATE TABLE IF NOT EXISTS phases (
    id BLOB PRIMARY KEY,
    project_id BLOB NOT NULL,
    name TEXT NOT NULL CHECK (name IN ('prompt','fix','hardening')),
    created_at TEXT NOT NULL DEFAULT (datetime('now','subsec')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Project context files (uploads)
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

-- Task context files (uploads)
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

-- Attempt artifacts directory metadata
CREATE TABLE IF NOT EXISTS attempt_artifacts (
    id BLOB PRIMARY KEY,
    attempt_id BLOB NOT NULL,
    kind TEXT NOT NULL,
    path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now','subsec')),
    FOREIGN KEY (attempt_id) REFERENCES task_attempts(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_attempt_artifacts_attempt ON attempt_artifacts(attempt_id);

-- Add attempt->phase FK and telemetry fields
ALTER TABLE task_attempts ADD COLUMN phase_id BLOB;
ALTER TABLE task_attempts ADD COLUMN agent_profile TEXT; -- e.g., gpt5 (Thinking)
ALTER TABLE task_attempts ADD COLUMN prompt_tokens INTEGER;
ALTER TABLE task_attempts ADD COLUMN completion_tokens INTEGER;
ALTER TABLE task_attempts ADD COLUMN cold_sec INTEGER;
ALTER TABLE task_attempts ADD COLUMN warm_sec INTEGER;
ALTER TABLE task_attempts ADD COLUMN cache_hit_count INTEGER;
ALTER TABLE task_attempts ADD COLUMN scope_pass INTEGER DEFAULT 0;
ALTER TABLE task_attempts ADD COLUMN dep_pass INTEGER DEFAULT 0;
ALTER TABLE task_attempts ADD COLUMN api_pass INTEGER DEFAULT 0;
ALTER TABLE task_attempts ADD COLUMN det_pass INTEGER DEFAULT 0;
ALTER TABLE task_attempts ADD COLUMN kpi_pass INTEGER DEFAULT 0;
