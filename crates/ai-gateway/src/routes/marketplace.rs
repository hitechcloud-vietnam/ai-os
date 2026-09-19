use axum::{Json, Router, routing::get};
use serde_json::json;
use sqlx::PgPool;

#[allow(dead_code)]
pub fn router() -> Router<PgPool> {
    Router::<PgPool>::new()
        .route("/marketplace/search", get(search))
        .route("/marketplace/featured", get(featured))
}

pub async fn search() -> Json<serde_json::Value> {
    Json(json!({"data": [], "total": 0, "message": "Marketplace — Phase 2"}))
}

pub async fn featured() -> Json<serde_json::Value> {
    Json(json!({"data": [], "message": "Featured — Phase 2"}))
}
