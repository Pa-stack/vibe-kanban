pub mod prompt;
pub mod patch;
pub mod apply;
pub mod test;
pub mod artifacts;

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct OrchestratorConfig {
    pub cache_dir: std::path::PathBuf,
    pub artifacts_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct Orchestrator;

impl Orchestrator {
    pub fn new() -> Self { Self }

    pub async fn run_attempt(&self, _cfg: OrchestratorConfig) -> Result<()> {
        // Placeholder wire-up; real logic will: assemble prompt, call agent, parse PATCH, apply, run tests twice, write artifacts
        Ok(())
    }
}
