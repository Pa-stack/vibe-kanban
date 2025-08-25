-- Unique index to guarantee deterministic phase_id
CREATE UNIQUE INDEX IF NOT EXISTS ux_phases_phase_id ON phases(phase_id);
