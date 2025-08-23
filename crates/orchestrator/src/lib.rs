pub mod prompt; pub mod patch; pub mod apply; pub mod test; pub mod artifacts;

#[derive(Clone, Default)]
pub struct OrchestratorConfig { pub cache_dir: std::path::PathBuf, pub artifacts_dir: std::path::PathBuf }

#[derive(Clone)]
pub struct Orchestrator;

impl Orchestrator { pub fn new() -> Self { Self } pub fn run_attempt(&self, _cfg: OrchestratorConfig) -> Result<(), String> { Ok(()) } }
