use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct RunOutcome {
    pub cold_sec: f64,
    pub warm_sec: f64,
    pub cache_hit_count: u32,
    pub snippets: String,
}

fn run_once(work_dir: &std::path::Path, cmd: &str, timeout_s: u64) -> Result<(f64, String), String> {
    let shell = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
    let start = Instant::now();
    let mut child = std::process::Command::new(shell.0)
        .arg(shell.1)
        .arg(cmd)
        .current_dir(work_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let to = std::time::Duration::from_secs(timeout_s);
    // Naive timeout loop
    loop {
        if let Ok(Some(_)) = child.try_wait() { break; }
        if start.elapsed() > to { let _ = child.kill(); return Err("test run timed out".to_string()); }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let secs = start.elapsed().as_secs_f64();
    let text = String::from_utf8_lossy(&output.stdout).to_string() +
        &String::from_utf8_lossy(&output.stderr);
    Ok((secs, text))
}

pub fn run_two(work_dir: &std::path::Path, _cache_dir: &std::path::Path, test_cmd: Option<&str>) -> Result<RunOutcome, String> {
    let cmd = test_cmd.unwrap_or("cargo test --workspace");
    let (cold_s, cold_out) = run_once(work_dir, cmd, 900)?;
    let (warm_s, warm_out) = run_once(work_dir, cmd, 600)?;
    let mut cache_hits = 0u32;
    for line in cold_out.lines().chain(warm_out.lines()) {
        if line.to_uppercase().contains("CACHE_HIT") { cache_hits += 1; }
    }
    // very small digest: sum of bytes as hex for both runs
    let sum_cold: u64 = cold_out.as_bytes().iter().fold(0u64, |acc, b| acc.wrapping_add(*b as u64));
    let sum_warm: u64 = warm_out.as_bytes().iter().fold(0u64, |acc, b| acc.wrapping_add(*b as u64));
    let digest_cold = format!("OUTPUT_DIGEST_COLD={:016x}", sum_cold);
    let digest_warm = format!("OUTPUT_DIGEST_WARM={:016x}", sum_warm);
    let mut lines = Vec::new();
    if cache_hits>0 { lines.push("CACHE_HIT".to_string()); }
    lines.push(digest_cold);
    lines.push(digest_warm);
    let snippets = lines.join("\n");
    Ok(RunOutcome { cold_sec: cold_s, warm_sec: warm_s, cache_hit_count: cache_hits, snippets })
}
