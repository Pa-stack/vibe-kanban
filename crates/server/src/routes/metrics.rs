use axum::{routing::get, Router, response::Json as ResponseJson};
use serde::Serialize;
use utils::response::ApiResponse;
use crate::DeploymentImpl;

#[derive(Serialize)]
struct MetricsSummary { runs: u64, pass_rate: f32 }

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new().route("/metrics/summary", get(get_summary))
}

async fn get_summary() -> ResponseJson<ApiResponse<MetricsSummary>> {
    ResponseJson(ApiResponse::success(MetricsSummary { runs: 0, pass_rate: 0.0 }))
}
