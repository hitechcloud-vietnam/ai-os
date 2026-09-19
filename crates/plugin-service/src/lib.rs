use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use hitechcloud_common::{errors::AppError, models::*};
use sqlx::PgPool;

pub fn plugin_router(_pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/", get(list_plugins).post(create_plugin))
        .route(
            "/{id}",
            get(get_plugin).put(update_plugin).delete(delete_plugin),
        )
        .route("/{id}/versions", get(get_plugin_versions))
}

pub async fn list_plugins(
    State(pool): State<PgPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Plugin>>, AppError> {
    let per_page = q.per_page.unwrap_or(20);
    let offset = (q.page.unwrap_or(1) - 1) * per_page;

    let plugins = sqlx::query_as::<_, Plugin>(
        r#"SELECT id, owner_org_id, name, description
           FROM plugins ORDER BY name LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&pool)
    .await?;
    Ok(Json(plugins))
}

pub async fn get_plugin(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Plugin>, AppError> {
    let plugin = sqlx::query_as::<_, Plugin>(
        r#"SELECT id, owner_org_id, name, description FROM plugins WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' not found", id)))?;
    Ok(Json(plugin))
}

pub async fn create_plugin(
    State(pool): State<PgPool>,
    Json(req): Json<CreatePluginRequest>,
) -> Result<Json<Plugin>, AppError> {
    let id = req.name.to_lowercase().replace(' ', "-");
    let plugin = sqlx::query_as::<_, Plugin>(
        r#"INSERT INTO plugins (id, name, description)
           VALUES ($1, $2, $3)
           RETURNING id, owner_org_id, name, description"#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.description)
    .fetch_one(&pool)
    .await?;
    Ok(Json(plugin))
}

pub async fn update_plugin(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(req): Json<CreatePluginRequest>,
) -> Result<Json<Plugin>, AppError> {
    let plugin = sqlx::query_as::<_, Plugin>(
        r#"UPDATE plugins SET name=$2, description=$3 WHERE id=$1
           RETURNING id, owner_org_id, name, description"#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.description)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Plugin '{}' not found", id)))?;
    Ok(Json(plugin))
}

pub async fn delete_plugin(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM plugins WHERE id = $1")
        .bind(&id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Plugin '{}' not found", id)));
    }
    Ok(Json(serde_json::json!({"deleted": true, "id": id})))
}

pub async fn get_plugin_versions(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Vec<PluginVersion>>, AppError> {
    let versions = sqlx::query_as::<_, PluginVersion>(
        r#"SELECT plugin_id, version, manifest, package_hash, signature, status
           FROM plugin_versions WHERE plugin_id = $1 ORDER BY version DESC"#,
    )
    .bind(&id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(versions))
}
