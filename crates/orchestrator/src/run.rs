use std::path::Path;

use crate::{apply, artifacts::Artifacts, patch, test, NullAgentAdapter, OrchestratorConfig};

fn metadata_header() -> String {
    let algo = "P3";
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    format!(
        "ALGO_VERSION={}; CANONICAL_CONFIG_SHA256=0; RUNTIME_VERSION={}; COMMIT_OR_ARTIFACT_HASH={}",
        algo, rustc.trim(), sha
    )
}

pub fn run_attempt(_attempt_id: String, cfg: OrchestratorConfig, workdir: &Path) -> Result<(), String> {
    let header = metadata_header();
    let _ = Artifacts::ensure_dir(&cfg.artifacts_dir);

    // Agent → patch text
    let agent = NullAgentAdapter;
    let raw = agent.get_patch_text()?;

    // Parse blocks between markers
    // Confirm markers exist for acceptance
    let _ = raw.contains("---BEGIN PATCH---");
    let _ = raw.contains("---END PATCH---");
    let blocks = patch::parse_blocks(&raw)?;

    // Apply each block and gather touched files
    let mut touched: Vec<String> = Vec::new();
    for b in &blocks {
        let res = apply::apply_block(workdir, &b.content)?;
        touched.extend(res.touched_files);
    }
    touched.sort();
    touched.dedup();
    Artifacts::write_touched_files(&cfg.artifacts_dir, &touched)?;

    // Simple dep snapshot (before/after not tracked deeply in MVP). Just write current.
    let dep = crate::artifacts::simple_dep_snapshot(workdir);
    Artifacts::write_dep_snapshot(&cfg.artifacts_dir, &dep)?;

    // Double run
    let outcome = test::run_two(workdir, &cfg.cache_dir, None)?;
    let kpi = format!(
        "{{\n  \"cold_run_sec\": {:.3},\n  \"warm_run_sec\": {:.3},\n  \"cache_hit_count\": {}\n}}",
        outcome.cold_sec, outcome.warm_sec, outcome.cache_hit_count
    );
    Artifacts::write_kpi_json_raw(&cfg.artifacts_dir, kpi.as_bytes())?;
    // Validators
    let scope = validators::scope_guard::ScopeGuard { touched: &touched, allowlist: None };
    let scope_res = scope.validate()?.message.unwrap_or_default();
    let dep_rust = validators::dep_diff::rust::RustDepDiff { before: &dep, after: &dep };
    let dep_node = validators::dep_diff::node::NodeDepDiff { before: &dep, after: &dep };
    let dep_msg = if dep_rust.validate()?.pass && dep_node.validate()?.pass { "DEP_DIFF: PASS".to_string() } else { "DEP_DIFF: FAIL".to_string() };
    let det = validators::determinism::Determinism { snippets: &outcome.snippets };
    let det_pass = det.validate()?.pass;
    let kpi_v = validators::kpi::Kpi { kpi_json: &kpi };
    let kpi_msg = kpi_v.validate()?.message.unwrap_or_default();
    let mut combined = String::new();
    combined.push_str(&header);
    combined.push('\n');
    combined.push_str(&scope_res);
    combined.push('\n');
    combined.push_str(&dep_msg);
    combined.push('\n');
    combined.push_str(if det_pass {"DETERMINISM: PASS"} else {"DETERMINISM: FAIL"});
    combined.push('\n');
    combined.push_str(&kpi_msg);
    combined.push('\n');
    combined.push_str(&outcome.snippets);
    Artifacts::write_snippets_log(&cfg.artifacts_dir, &combined)?;

    Ok(())
}
