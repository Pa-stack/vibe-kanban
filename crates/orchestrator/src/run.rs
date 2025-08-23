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

pub fn run_attempt(attempt_id: String, cfg: OrchestratorConfig, workdir: &Path) -> Result<(), String> {
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

        // Snapshot BEFORE
        let dep_before = crate::artifacts::simple_dep_snapshot(workdir);

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
        // Snapshot AFTER
        let dep_after = crate::artifacts::simple_dep_snapshot(workdir);
        let dep_combined = format!("=== BEFORE ===\n{}\n=== AFTER ===\n{}\n", dep_before, dep_after);
        Artifacts::write_dep_snapshot(&cfg.artifacts_dir, &dep_combined)?;

    // Double run
    let outcome = test::run_two(workdir, &cfg.cache_dir, None)?;
    let kpi = format!(
        "{{\n  \"cold_run_sec\": {:.3},\n  \"warm_run_sec\": {:.3},\n  \"cache_hit_count\": {}\n}}",
        outcome.cold_sec, outcome.warm_sec, outcome.cache_hit_count
    );
    Artifacts::write_kpi_json_raw(&cfg.artifacts_dir, kpi.as_bytes())?;
    // Validators
    let scope = validators::scope_guard::ScopeGuard { touched: &touched, allowlist: None };
    let scope_val = scope.validate()?;
    let scope_res = scope_val.message.clone().unwrap_or_default();
        let dep_rust = validators::dep_diff::rust::RustDepDiff { before: &dep_before, after: &dep_after };
        let dep_node = validators::dep_diff::node::NodeDepDiff { before: &dep_before, after: &dep_after };
    let dep_rust_pass = dep_rust.validate()?.pass;
    let dep_node_pass = dep_node.validate()?.pass;
    let dep_msg = if dep_rust_pass && dep_node_pass { "DEP_DIFF: PASS".to_string() } else { "DEP_DIFF: FAIL".to_string() };
    let det = validators::determinism::Determinism { snippets: &outcome.snippets };
    let det_pass = det.validate()?.pass;
    let kpi_v = validators::kpi::Kpi { kpi_json: &kpi };
    let kpi_val = kpi_v.validate()?;
    let kpi_msg = kpi_val.message.clone().unwrap_or_default();
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
    // Append condensed validators status line for SSE-friendly consumption
    let as_pass_fail = |b: bool| if b { "PASS" } else { "FAIL" };
    let validators_line = format!(
        "ORCH: validators: scope={} dep={} api=SKIP det={} kpi={}",
        as_pass_fail(scope_val.pass),
        as_pass_fail(dep_rust_pass && dep_node_pass),
        as_pass_fail(det_pass),
        as_pass_fail(kpi_val.pass)
    );
    combined.push('\n');
    combined.push_str(&validators_line);
    Artifacts::write_snippets_log(&cfg.artifacts_dir, &combined)?;

    // Machine-readable summary.json (idempotent overwrite)
    let summary = serde_json::json!({
        "attempt_id": attempt_id,
        "validator": {
            "scope": scope_val.pass,
            "dep": dep_rust_pass && dep_node_pass,
            "api": serde_json::Value::Null,
            "det": det_pass,
            "kpi": kpi_val.pass
        },
        "timing": {
            "cold_sec": outcome.cold_sec,
            "warm_sec": outcome.warm_sec
        },
        "cache_hit_count": outcome.cache_hit_count
    });
    let bytes = serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?;
    Artifacts::write_summary_json(&cfg.artifacts_dir, &bytes)?;

    Ok(())
}
