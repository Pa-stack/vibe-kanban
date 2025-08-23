use axum::{routing::get, Router, response::Json as ResponseJson};
use utils::response::ApiResponse;
use crate::DeploymentImpl;

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new().route("/metrics/summary", get(get_summary))
}

async fn get_summary() -> ResponseJson<ApiResponse<&'static str>> {
    ResponseJson(ApiResponse::success("ok"))
}
