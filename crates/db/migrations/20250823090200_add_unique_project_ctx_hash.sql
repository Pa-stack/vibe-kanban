-- Atomic dedup for project context files
-- Forward-only, idempotent
CREATE UNIQUE INDEX IF NOT EXISTS ux_project_context_files_scope_hash
ON project_context_files(project_id, sha256);
