export type Phase = { task_id: string; phase_id: string; type: 'prompt'|'fix'|'hardening'; status: 'idle'|'running'|'pass'|'fail'; allowlist: any[]; denylist: any[]; agent_override: string|null; warm_kpi_budget: number|null; created_at?: string; updated_at?: string };

export async function createPhase(taskId: string): Promise<Phase> {
  const r = await fetch(`/api/tasks/${taskId}/phases`, { method:'POST', headers:{'content-type':'application/json'}, body: JSON.stringify({})});
  const j = await r.json();
  return j.data as Phase;
}

export async function updatePhase(phaseId: string, patch: Partial<Phase>): Promise<Partial<Phase>> {
  const r = await fetch(`/api/phases/${phaseId}`, { method:'PATCH', headers:{'content-type':'application/json'}, body: JSON.stringify(patch)});
  const j = await r.json();
  return j.data as Partial<Phase>;
}

export type ArtifactsSummary = { [k:string]: string|any, summary?: { validator?: { scope?: boolean; dep?: boolean; api?: boolean|null; det?: boolean; kpi?: boolean } } };
export async function getArtifactsSummary(attemptId: string): Promise<ArtifactsSummary> {
  const r = await fetch(`/api/attempts/${attemptId}/artifacts`);
  const j = await r.json();
  return j.data as ArtifactsSummary;
}
