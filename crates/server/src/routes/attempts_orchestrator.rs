use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
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
) -> ResponseJson<ApiResponse<&'static str>> {
    let attempt_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return ResponseJson(ApiResponse::error("Invalid attempt id")),
    };

    // Resolve attempt, task, project
    let pool = &deployment.db().pool;
    let Some(attempt) = TaskAttempt::find_by_id(pool, attempt_id).await.ok().flatten() else {
        return ResponseJson(ApiResponse::error("Attempt not found"));
    };
    let Some(task) = attempt.parent_task(pool).await.ok().flatten() else {
        return ResponseJson(ApiResponse::error("Task not found"));
    };
    let Some(project) = task.parent_project(pool).await.ok().flatten() else {
        return ResponseJson(ApiResponse::error("Project not found"));
    };

    // Feature flag: use workspace_dir as data dir; if missing, reject
    let vk_dir_env = std::env::var("VK_DATA_DIR").ok().map(PathBuf::from);
    let cfg_read = deployment.config().read().await;
    let data_dir = vk_dir_env.or_else(|| cfg_read.workspace_dir.as_ref().map(PathBuf::from));
    let Some(data_dir) = data_dir else {
        return ResponseJson(ApiResponse::error(
            "Orchestrator disabled (no data dir configured)",
        ));
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

    ResponseJson(ApiResponse::success("started"))
}

async fn get_artifacts(
    Path(id): Path<String>,
) -> ResponseJson<ApiResponse<Vec<String>>> {
    let Some(data_dir) = std::env::var("VK_DATA_DIR").ok().map(PathBuf::from) else {
        return ResponseJson(ApiResponse::success(vec![]));
    };
    let dir = data_dir.join("artifacts").join(id);
    let list = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flat_map(|rd| rd.filter_map(|e| e.ok()))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    ResponseJson(ApiResponse::success(list))
}
