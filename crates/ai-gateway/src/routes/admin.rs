use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::json;
use sqlx::PgPool;

#[allow(dead_code)]
pub fn router() -> Router<PgPool> {
    Router::<PgPool>::new()
        .route("/admin/orgs", get(list_orgs))
        .route("/admin/orgs/{org_id}/members", get(get_members))
        .route("/admin/orgs/{org_id}/audit-logs", get(get_audit_logs))
}

pub async fn list_orgs(State(pool): State<PgPool>) -> Json<serde_json::Value> {
    let orgs: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id::text, name, plan FROM organizations ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    Json(json!({
        "data": orgs.iter().map(|(id, name, plan)| json!({"id": id, "name": name, "plan": plan})).collect::<Vec<_>>(),
    }))
}

pub async fn get_members(
    State(pool): State<PgPool>,
    Path(org_id): Path<String>,
) -> Json<serde_json::Value> {
    let members: Vec<(String, String)> = sqlx::query_as(
        "SELECT id::text, email FROM users WHERE org_id::text = $1 ORDER BY email LIMIT 100",
    )
    .bind(&org_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    Json(json!({
        "data": members.iter().map(|(id, email)| json!({"id": id, "email": email})).collect::<Vec<_>>(),
    }))
}

pub async fn get_audit_logs(
    State(pool): State<PgPool>,
    Path(org_id): Path<String>,
) -> Json<serde_json::Value> {
    let logs: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, action, result_status FROM audit_logs WHERE org_id::text = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(&org_id).fetch_all(&pool).await.unwrap_or_default();
    Json(json!({
        "data": logs.iter().map(|(id, action, status)| json!({"id": id, "action": action, "status": status})).collect::<Vec<_>>(),
    }))
}
