use axum::{routing::{get, post}, Router, extract::Path, response::Json as ResponseJson};
use utils::response::ApiResponse;
use crate::DeploymentImpl;

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new().route("/tasks/:id/uploads", post(upload).get(list))
}

async fn upload(Path(_id): Path<String>) -> ResponseJson<ApiResponse<&'static str>> {
    ResponseJson(ApiResponse::success("ok"))
}

async fn list(Path(_id): Path<String>) -> ResponseJson<ApiResponse<Vec<String>>> {
    ResponseJson(ApiResponse::success(vec![]))
}
