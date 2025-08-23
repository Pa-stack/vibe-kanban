-- P4 phases additive fields
ALTER TABLE phases ADD COLUMN task_id BLOB;
ALTER TABLE phases ADD COLUMN phase_id TEXT;
ALTER TABLE phases ADD COLUMN type TEXT;
ALTER TABLE phases ADD COLUMN status TEXT;
ALTER TABLE phases ADD COLUMN allowlist TEXT;
ALTER TABLE phases ADD COLUMN denylist TEXT;
ALTER TABLE phases ADD COLUMN agent_override TEXT;
ALTER TABLE phases ADD COLUMN warm_kpi_budget REAL;
ALTER TABLE phases ADD COLUMN created_at TEXT;
ALTER TABLE phases ADD COLUMN updated_at TEXT;

-- Index for deterministic listing
CREATE INDEX IF NOT EXISTS idx_phases_task_created ON phases(task_id, created_at);
