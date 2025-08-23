use axum::{extract::{Path, State}, response::Json as ResponseJson, routing::{post, patch, get}, Router, Json, middleware::from_fn_with_state};
use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;
use sqlx::Row;
use crate::{DeploymentImpl, middleware::load_task_middleware};
use utils::response::ApiResponse;

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/tasks/:id/phases", post(create_phase).get(list_phases)
            .layer(from_fn_with_state(deployment.clone(), load_task_middleware)))
        .route("/phases/:id", patch(update_phase))
}

async fn create_phase(State(deployment): State<DeploymentImpl>, Path(task_id): Path<String>) -> ResponseJson<ApiResponse<serde_json::Value>> {
    let Ok(task_uuid) = Uuid::parse_str(&task_id) else { return ResponseJson(ApiResponse::error("invalid_task_id")); };
    let phase_id = format!("P4-2025-08-23-{}", &Uuid::new_v4().to_string()[..8]);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_else(|_| "0".to_string());
    let pool = &deployment.db().pool;
    let _ = sqlx::query("INSERT INTO phases (task_id, phase_id, type, status, allowlist, denylist, agent_override, warm_kpi_budget, created_at, updated_at) VALUES (?1, ?2, 'prompt', 'idle', '[]', '[]', NULL, NULL, ?3, ?3)")
        .bind(task_uuid)
        .bind(&phase_id)
        .bind(&now)
        .execute(pool).await;
    ResponseJson(ApiResponse::success(json!({
        "task_id": task_uuid,
        "phase_id": phase_id,
        "type": "prompt",
        "status": "idle",
        "allowlist": [],
        "denylist": [],
        "agent_override": null,
        "warm_kpi_budget": null,
        "created_at": now,
        "updated_at": now
    })))
}

#[derive(serde::Deserialize)]
struct PhasePatch { status: Option<String>, allowlist: Option<serde_json::Value>, denylist: Option<serde_json::Value>, agent_override: Option<Option<String>>, warm_kpi_budget: Option<Option<f64>>, r#type: Option<String> }

async fn update_phase(State(deployment): State<DeploymentImpl>, Path(phase_id): Path<String>, Json(p): Json<PhasePatch>) -> (StatusCode, ResponseJson<ApiResponse<serde_json::Value>>) {
    let pool = &deployment.db().pool;
    // Enforce scoping: phase must belong to a task accessible to caller
    let exists = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(1) FROM phases p JOIN tasks t ON p.task_id = t.id WHERE p.phase_id = ?"
    )
    .bind(&phase_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if exists == 0 {
        // Not found or not accessible
    return (StatusCode::NOT_FOUND, ResponseJson(ApiResponse::error("not_found")));
    }
    // Validate enums when present
    if let Some(ref typ) = p.r#type {
        if !(matches!(typ.as_str(), "prompt" | "fix" | "hardening")) {
            return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("invalid_type")));
        }
    }
    if let Some(ref status) = p.status {
        if !(matches!(status.as_str(), "idle" | "running" | "pass" | "fail")) {
            return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("invalid_status")));
        }
    }
    let mut sets: Vec<&str> = Vec::new();
    if p.status.is_some() { sets.push("status = ?"); }
    if p.allowlist.is_some() { sets.push("allowlist = ?"); }
    if p.denylist.is_some() { sets.push("denylist = ?"); }
    if p.agent_override.is_some() { sets.push("agent_override = ?"); }
    if p.warm_kpi_budget.is_some() { sets.push("warm_kpi_budget = ?"); }
    if p.r#type.is_some() { sets.push("type = ?"); }
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_else(|_| "0".to_string());
    sets.push("updated_at = ?");
    let sql = format!("UPDATE phases SET {} WHERE phase_id = ?", sets.join(", "));
    let mut q = sqlx::query(&sql);
    if let Some(v) = p.status { q = q.bind(v); }
    if let Some(v) = p.allowlist {
        // Canonicalize and cap payload size (16 KiB)
        let s = serde_json::to_string(&v).unwrap_or_else(|_| "[]".to_string());
        if s.len() > 16 * 1024 { return (StatusCode::PAYLOAD_TOO_LARGE, ResponseJson(ApiResponse::error("payload_too_large"))); }
        q = q.bind(s);
    }
    if let Some(v) = p.denylist {
        let s = serde_json::to_string(&v).unwrap_or_else(|_| "[]".to_string());
        if s.len() > 16 * 1024 { return (StatusCode::PAYLOAD_TOO_LARGE, ResponseJson(ApiResponse::error("payload_too_large"))); }
        q = q.bind(s);
    }
    if let Some(v) = p.agent_override { q = q.bind(v); }
    if let Some(v) = p.warm_kpi_budget { q = q.bind(v); }
    if let Some(v) = p.r#type { q = q.bind(v); }
    q = q.bind(&now).bind(&phase_id);
    let _ = q.execute(pool).await;
    (StatusCode::OK, ResponseJson(ApiResponse::success(json!({
        "phase_id": phase_id,
        "status": p.status,
        "warm_kpi_budget": p.warm_kpi_budget,
    }))))
}

async fn list_phases(
    State(deployment): State<DeploymentImpl>,
    Path(task_id): Path<String>,
) -> ResponseJson<ApiResponse<serde_json::Value>> {
    let Ok(task_uuid) = Uuid::parse_str(&task_id) else {
        return ResponseJson(ApiResponse::error("invalid_task_id"));
    };
    let pool = &deployment.db().pool;
    let rows = sqlx::query(
        "SELECT phase_id, type, status, allowlist, denylist, agent_override, warm_kpi_budget, created_at, updated_at FROM phases WHERE task_id = ?1 ORDER BY created_at ASC, phase_id ASC",
    )
    .bind(task_uuid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let list: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            let phase_id: String = r.get::<String, _>(0);
            let typ: String = r.get::<String, _>(1);
            let status: String = r.get::<String, _>(2);
            let allowlist: String = r.get::<String, _>(3);
            let denylist: String = r.get::<String, _>(4);
            let agent_override: Option<String> = r.get::<Option<String>, _>(5);
            let warm_kpi_budget: Option<f64> = r.get::<Option<f64>, _>(6);
            let created_at: String = r.get::<String, _>(7);
            let updated_at: String = r.get::<String, _>(8);
            serde_json::json!({
                "phase_id": phase_id,
                "type": typ,
                "status": status,
                "allowlist": serde_json::from_str::<serde_json::Value>(&allowlist).unwrap_or(serde_json::json!([])),
                "denylist": serde_json::from_str::<serde_json::Value>(&denylist).unwrap_or(serde_json::json!([])),
                "agent_override": agent_override,
                "warm_kpi_budget": warm_kpi_budget,
                "created_at": created_at,
                "updated_at": updated_at,
            })
        })
        .collect();
    ResponseJson(ApiResponse::success(serde_json::json!(list)))
}
