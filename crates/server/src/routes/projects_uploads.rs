use std::{env, fs, io::Write, path::{Path, PathBuf}};

use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    middleware::from_fn_with_state,
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Extension, Router, body::Body,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{middleware::load_project_middleware, util::{hash::sha256_hex, mime::detect_mime}, DeploymentImpl};

const MAX_SIZE: usize = 30 * 1024 * 1024; // 30MB

#[derive(Debug, Serialize)]
struct UploadMeta {
    filename: String,
    mime: String,
    size: i64,
    content_hash: String,
    stored_at: String,
    extracted_text: bool,
    dedup: bool,
}

#[derive(FromRow)]
struct ProjectCtxRow {
    id: Uuid,
    project_id: Uuid,
    filename: String,
    mime: String,
    size_bytes: i64,
    sha256: String,
    stored_path: String,
    created_at: DateTime<Utc>,
}

fn data_root() -> PathBuf {
    if let Ok(dir) = env::var("VK_DATA_DIR") {
        PathBuf::from(dir)
    } else {
        // fallback to app data dir
        utils::assets::asset_dir()
    }
}

fn normalize_filename(name: &str) -> String {
    let candidate = name.replace(['\\', '/', ':'], "_");
    let s: String = candidate
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || ['.', '-', '_'].contains(c))
        .collect();
    if s.is_empty() { "upload".to_string() } else { s }
}

fn allowed_mime(mime: &str) -> bool {
    matches!(mime,
        "application/pdf" | "text/plain" | "text/markdown" | "application/json" | "text/csv" | "image/png" | "image/jpeg"
    )
}

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
    .route("/projects/:id/uploads", post(upload).get(list).layer(DefaultBodyLimit::max(MAX_SIZE)))
    .route("/projects/:id/uploads/:hash/:file", get(download))
        .layer(from_fn_with_state(
            deployment.clone(),
            load_project_middleware,
        ))
}

pub async fn upload(
    State(deployment): State<DeploymentImpl>,
    Extension(project): Extension<db::models::project::Project>,
    mut multipart: Multipart,
) -> Result<axum::Json<ApiResponse<UploadMeta>>, crate::error::ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("file") { continue; }
        let orig_name = field.file_name().unwrap_or("upload.bin");
        let bytes = field.bytes().await?;
        if bytes.len() > MAX_SIZE {
            return Ok(axum::Json(ApiResponse::error("payload_too_large")));
        }
        let fname = normalize_filename(orig_name);
        let mime = detect_mime(&bytes, &fname).unwrap_or_else(|| "application/octet-stream".into());
        if !allowed_mime(&mime) {
            return Ok(axum::Json(ApiResponse::error("mime_not_allowed")));
        }
        let hash = sha256_hex(&bytes);
        let root = data_root().join("blob").join(&hash);
        fs::create_dir_all(&root)?;
        let stored_path_rel = format!("blob/{hash}/{fname}");
        let full_path = root.join(&fname);
        if !full_path.exists() {
            let mut f = fs::File::create(&full_path)?;
            f.write_all(&bytes)?;
        }
        // Sidecar for PDFs
        let sidecar = data_root().join("blob").join(format!("{hash}.txt"));
        let mut extracted = false;
        if mime == "application/pdf" {
            if !sidecar.exists() {
                // Create empty stub (no extractor available)
                let _ = fs::File::create(&sidecar);
            }
            extracted = fs::metadata(&sidecar).map(|m| m.len() > 0).unwrap_or(false);
        } else {
            extracted = sidecar.exists() && fs::metadata(&sidecar).map(|m| m.len() > 0).unwrap_or(false);
        }

        // Dedup by (project_id, sha256)
        let existing = sqlx::query_as!(
            ProjectCtxRow,
            r#"SELECT id as "id: Uuid", project_id as "project_id: Uuid", filename, mime, size_bytes, sha256, stored_path, created_at as "created_at: DateTime<Utc>" FROM project_context_files WHERE project_id = ?1 AND sha256 = ?2 LIMIT 1"#,
            project.id,
            hash
        )
        .fetch_optional(&deployment.db().pool)
        .await?;

        let (row, dedup) = if let Some(r) = existing { (r, true) } else {
            let id = Uuid::new_v4();
            let r = sqlx::query_as!(
                ProjectCtxRow,
                r#"INSERT INTO project_context_files (id, project_id, filename, mime, size_bytes, sha256, stored_path) VALUES (?1,?2,?3,?4,?5,?6,?7) RETURNING id as "id: Uuid", project_id as "project_id: Uuid", filename, mime, size_bytes, sha256, stored_path, created_at as "created_at: DateTime<Utc>""#,
                id,
                project.id,
                fname,
                mime,
                bytes.len() as i64,
                hash,
                stored_path_rel
            )
            .fetch_one(&deployment.db().pool)
            .await?;
            (r, false)
        };

        let meta = UploadMeta {
            filename: row.filename,
            mime: row.mime,
            size: row.size_bytes,
            content_hash: row.sha256,
            stored_at: row.stored_path,
            extracted_text: extracted,
            dedup,
        };
        return Ok(axum::Json(ApiResponse::success(meta)));
    }
    Ok(axum::Json(ApiResponse::error("file_missing")))
}

pub async fn list(
    State(deployment): State<DeploymentImpl>,
    Extension(project): Extension<db::models::project::Project>,
) -> Result<axum::Json<ApiResponse<Vec<UploadMeta>>>, crate::error::ApiError> {
    let rows: Vec<ProjectCtxRow> = sqlx::query_as!(
        ProjectCtxRow,
        r#"SELECT id as "id: Uuid", project_id as "project_id: Uuid", filename, mime, size_bytes, sha256, stored_path, created_at as "created_at: DateTime<Utc>" FROM project_context_files WHERE project_id = ?1 ORDER BY created_at ASC, filename ASC"#,
        project.id
    )
    .fetch_all(&deployment.db().pool)
    .await?;

    let metas = rows
        .into_iter()
        .map(|r| {
            let sidecar = data_root().join("blob").join(format!("{}.txt", r.sha256));
            let extracted = sidecar.exists() && fs::metadata(&sidecar).map(|m| m.len() > 0).unwrap_or(false);
            UploadMeta {
                filename: r.filename,
                mime: r.mime,
                size: r.size_bytes,
                content_hash: r.sha256,
                stored_at: r.stored_path,
                extracted_text: extracted,
                dedup: false,
            }
        })
        .collect();

    Ok(axum::Json(ApiResponse::success(metas)))
}

pub async fn download(
    State(deployment): State<DeploymentImpl>,
    Extension(project): Extension<db::models::project::Project>,
    axum::extract::Path((hash, file)): axum::extract::Path<(String, String)>,
) -> Result<Response, crate::error::ApiError> {
    // Verify row exists for project/hash
    let exists = sqlx::query!(
        r#"SELECT 1 as "one!: i64" FROM project_context_files WHERE project_id = ?1 AND sha256 = ?2 LIMIT 1"#,
        project.id,
        hash
    )
    .fetch_optional(&deployment.db().pool)
    .await?
    .is_some();
    if !exists {
        let body = serde_json::json!({"error":"forbidden"}).to_string();
        let resp = Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();
        return Ok(resp);
    }
    let p = data_root().join("blob").join(&hash).join(&file);
    if !p.exists() {
        return Ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::json!({"error":"not_found"}).to_string()))
                .unwrap(),
        );
    }
    let data = tokio::fs::read(&p).await?;
    let mime = super::super::util::mime::detect_mime(&data, &file)
        .unwrap_or_else(|| "application/octet-stream".into());
    let body = Body::from(data);
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file),
        )
        .body(body)
        .unwrap();
    Ok(resp)
}
