use axum::{routing::{get, post}, Router, extract::Path, response::Json as ResponseJson};
use serde::Serialize;
use utils::response::ApiResponse;
use crate::DeploymentImpl;

#[derive(Serialize)]
struct UploadMeta { id: String, filename: String }

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/projects/:id/uploads", post(upload).get(list))
}

async fn upload(Path(_id): Path<String>) -> ResponseJson<ApiResponse<UploadMeta>> {
    ResponseJson(ApiResponse::success(UploadMeta { id: "0".into(), filename: "".into() }))
}

async fn list(Path(_id): Path<String>) -> ResponseJson<ApiResponse<Vec<UploadMeta>>> {
    ResponseJson(ApiResponse::success(vec![]))
}
