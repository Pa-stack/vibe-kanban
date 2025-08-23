use axum::{extract::Path, response::Json as ResponseJson, routing::{get, post}, Router};
use serde::Serialize;
use utils::response::ApiResponse;

use crate::DeploymentImpl;

#[derive(Serialize)]
struct OrchestratorKickoff { status: &'static str }

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/attempts/:id/run-orchestrator", post(run_orchestrator))
        .route("/attempts/:id/artifacts", get(get_artifacts))
}

async fn run_orchestrator(Path(_id): Path<String>) -> ResponseJson<ApiResponse<OrchestratorKickoff>> {
    ResponseJson(ApiResponse::success(OrchestratorKickoff { status: "queued" }))
}

async fn get_artifacts(Path(_id): Path<String>) -> ResponseJson<ApiResponse<Vec<String>>> {
    ResponseJson(ApiResponse::success(vec![]))
}
