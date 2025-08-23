#[derive(Debug, Clone, Default)]
pub struct ApplyResult {
    pub touched_files: Vec<String>,
}

pub fn apply_unified_diff(_repo_root: &std::path::Path, _diff: &str) -> Result<ApplyResult, String> {
    // TODO: leverage utils::diff helper if available; for now, return empty result
    Ok(ApplyResult { touched_files: vec![] })
}
