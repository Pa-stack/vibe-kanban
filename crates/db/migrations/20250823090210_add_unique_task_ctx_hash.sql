-- Atomic dedup for task context files
-- Forward-only, idempotent
CREATE UNIQUE INDEX IF NOT EXISTS ux_task_context_files_scope_hash
ON task_context_files(task_id, sha256);
