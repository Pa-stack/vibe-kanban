use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use axum::http::StatusCode;
use db::models::{task::Task, task_attempt::TaskAttempt};
use orchestrator::{artifacts::Artifacts, OrchestratorConfig};
use tokio::task;
use utils::response::ApiResponse;
use uuid::Uuid;
use crate::DeploymentImpl;

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/attempts/:id/run-orchestrator", post(run_orchestrator))
        .route("/attempts/:id/artifacts", get(get_artifacts))
}

async fn run_orchestrator(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<String>,
) -> (StatusCode, ResponseJson<serde_json::Value>) {
    let attempt_id = match Uuid::parse_str(&id) {
    Ok(u) => u,
    Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(serde_json::json!({"error": "invalid_attempt_id"}))),
    };

    // Resolve attempt, task, project
    let pool = &deployment.db().pool;
    let Some(attempt) = TaskAttempt::find_by_id(pool, attempt_id).await.ok().flatten() else {
        return (StatusCode::NOT_FOUND, ResponseJson(serde_json::json!({"error": "attempt_not_found"})));
    };
    let Some(task) = attempt.parent_task(pool).await.ok().flatten() else {
        return (StatusCode::NOT_FOUND, ResponseJson(serde_json::json!({"error": "task_not_found"})));
    };
    let Some(project) = task.parent_project(pool).await.ok().flatten() else {
        return (StatusCode::NOT_FOUND, ResponseJson(serde_json::json!({"error": "project_not_found"})));
    };

    // Feature flag: use workspace_dir as data dir; if missing, reject
    let vk_dir_env = std::env::var("VK_DATA_DIR").ok().map(PathBuf::from);
    let cfg_read = deployment.config().read().await;
    let data_dir = vk_dir_env.or_else(|| cfg_read.workspace_dir.as_ref().map(PathBuf::from));
    let Some(data_dir) = data_dir else {
        return (StatusCode::CONFLICT, ResponseJson(serde_json::json!({"error": "orchestrator_disabled"})));
    };

    let artifacts_dir = data_dir.join("artifacts").join(attempt_id.to_string());
    let cache_dir = data_dir.join("cache");

    // Spawn orchestrator run
    let workdir = project.git_repo_path.clone();
    let _ = deployment
        .events()
        .msg_store()
        .push_stdout(format!("ORCH: started attempt={}", attempt_id));
    task::spawn(async move {
        let cfg = OrchestratorConfig {
            cache_dir,
            artifacts_dir,
        };
        let _ = Artifacts::ensure_dir(&cfg.artifacts_dir);
    let _ = orchestrator::run::run_attempt(attempt_id.to_string(), cfg.clone(), &workdir);
    });

    (StatusCode::ACCEPTED, ResponseJson(serde_json::json!({"started": true})))
}

async fn get_artifacts(
    Path(id): Path<String>,
) -> ResponseJson<ApiResponse<serde_json::Value>> {
    let base = std::env::var("VK_DATA_DIR").ok().map(PathBuf::from);
    let dir = base.map(|b| b.join("artifacts").join(&id));
    let key = |name: &str| {
        dir.as_ref()
            .and_then(|d| {
                let p = d.join(name);
                if p.exists() { Some(p.to_string_lossy().to_string()) } else { None }
            })
    };
    let summary_path = key("summary.json");
    let summary_obj = summary_path.as_ref().and_then(|p| {
        std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
    }).unwrap_or_else(|| serde_json::json!({}));
    let payload = serde_json::json!({
        "touched_files.txt": key("touched_files.txt").unwrap_or_default(),
        "dep_snapshot.txt": key("dep_snapshot.txt").unwrap_or_default(),
        "kpi.json": key("kpi.json").unwrap_or_default(),
        "snippets.log": key("snippets.log").unwrap_or_default(),
        "summary.json": summary_path.unwrap_or_default(),
        "summary": summary_obj,
    });
    ResponseJson(ApiResponse::success(payload))
}
