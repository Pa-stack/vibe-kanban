use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct RunOutcome {
    pub cold_sec: u64,
    pub warm_sec: u64,
    pub cache_hit_count: u32,
}

pub async fn run_cold_warm(_work_dir: &std::path::Path, _cache_dir: &std::path::Path) -> Result<RunOutcome> {
    // TODO: actually run tests with cache; placeholder returns zeros
    tokio::time::sleep(Duration::from_millis(10)).await;
    Ok(RunOutcome { cold_sec: 0, warm_sec: 0, cache_hit_count: 0 })
}
