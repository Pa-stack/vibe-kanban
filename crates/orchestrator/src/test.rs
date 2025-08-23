use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct RunOutcome {
    pub cold_sec: u64,
    pub warm_sec: u64,
    pub cache_hit_count: u32,
}

pub fn run_cold_warm(_work_dir: &std::path::Path, _cache_dir: &std::path::Path) -> Result<RunOutcome, String> {
    let _ = Duration::from_millis(1); // placeholder
    Ok(RunOutcome { cold_sec: 0, warm_sec: 0, cache_hit_count: 0 })
}
