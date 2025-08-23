#[derive(Debug, Clone, Default)]
pub struct ApplyResult {
    pub touched_files: Vec<String>,
}

fn normalize_path(p: &str) -> Option<String> {
    if p.starts_with('/') || p.contains(':') || p.contains("..") { return None; }
    Some(p.trim_start_matches("./").to_string())
}

pub fn apply_block(repo_root: &std::path::Path, diff: &str) -> Result<ApplyResult, String> {
    // Collect touched files from diff headers
    let mut files = Vec::new();
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            if let Some(n) = normalize_path(rest) { files.push(n); }
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            if let Some(n) = normalize_path(rest) { files.push(n); }
        }
    }
    files.sort(); files.dedup();

    // Apply via git apply
    let status = std::process::Command::new("git")
        .arg("apply")
        .arg("--whitespace=nowarn")
        .arg("-p0")
        .current_dir(repo_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(diff.as_bytes())?;
            }
            child.wait()
        })
        .map_err(|e| e.to_string())?;
    if !status.success() { return Err("git apply failed".to_string()); }

    Ok(ApplyResult { touched_files: files })
}
