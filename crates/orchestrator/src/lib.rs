pub mod prompt;
pub mod patch;
pub mod apply;
pub mod test;
pub mod artifacts;
pub mod run;

#[derive(Clone, Default)]
pub struct OrchestratorConfig {
	pub cache_dir: std::path::PathBuf,
	pub artifacts_dir: std::path::PathBuf,
}

pub trait AgentAdapter: Send + Sync {
	fn get_patch_text(&self) -> Result<String, String>;
}

pub struct NullAgentAdapter;
impl AgentAdapter for NullAgentAdapter {
	fn get_patch_text(&self) -> Result<String, String> {
		let path = std::env::var("VK_FAKE_PATCH_PATH")
			.map_err(|_| "VK_FAKE_PATCH_PATH not set".to_string())?;
		std::fs::read_to_string(path).map_err(|e| e.to_string())
	}
}

#[derive(Clone)]
pub struct Orchestrator;
impl Orchestrator {
	pub fn new() -> Self {
		Self
	}
}
