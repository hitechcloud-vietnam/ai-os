use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use hitechcloud_common::{errors::AppError, models::*};
use sqlx::PgPool;

pub fn registry_router(_pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route(
            "/mcp-servers",
            post(create_mcp_server).get(list_mcp_servers),
        )
        .route(
            "/mcp-servers/{id}",
            get(get_mcp_server)
                .put(update_mcp_server)
                .delete(delete_mcp_server),
        )
        .route("/skills", post(create_skill).get(list_skills))
        .route(
            "/skills/{id}",
            get(get_skill).put(update_skill).delete(delete_skill),
        )
        .route("/skills/search", get(search_skills))
        .route("/plugins", post(create_plugin).get(list_plugins))
        .route(
            "/plugins/{id}",
            get(get_plugin).put(update_plugin).delete(delete_plugin),
        )
        .route("/plugins/{id}/versions", get(get_plugin_versions))
        .route(
            "/installations",
            post(create_installation).get(list_installations),
        )
        .route(
            "/installations/{id}",
            get(get_installation).delete(delete_installation),
        )
}

// ═══════════════════════════════════════════════════════════════
//  MCP Servers CRUD
// ═══════════════════════════════════════════════════════════════

pub async fn create_mcp_server(
    State(pool): State<PgPool>,
    Json(req): Json<CreateMcpServerRequest>,
) -> Result<Json<McpServer>, AppError> {
    let id = req.name.to_lowercase().replace(' ', "-");
    let visibility = req.visibility.unwrap_or_else(|| "public".to_string());

    let server = sqlx::query_as::<_, McpServer>(
        r#"INSERT INTO mcp_servers (id, name, transport, namespace, visibility)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, name, transport, namespace, visibility, owner_org_id, created_at"#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.transport)
    .bind(&req.namespace)
    .bind(&visibility)
    .fetch_one(&pool)
    .await?;

    Ok(Json(server))
}

pub async fn list_mcp_servers(
    State(pool): State<PgPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<McpServer>>, AppError> {
    let per_page = q.per_page.unwrap_or(20);
    let offset = (q.page.unwrap_or(1) - 1) * per_page;

    let servers = sqlx::query_as::<_, McpServer>(
        r#"SELECT id, name, transport, namespace, visibility, owner_org_id, created_at
           FROM mcp_servers ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    Ok(Json(servers))
}

pub async fn get_mcp_server(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<McpServer>, AppError> {
    let server = sqlx::query_as::<_, McpServer>(
        r#"SELECT id, name, transport, namespace, visibility, owner_org_id, created_at
           FROM mcp_servers WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("MCP server '{}' not found", id)))?;

    Ok(Json(server))
}

pub async fn update_mcp_server(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(req): Json<CreateMcpServerRequest>,
) -> Result<Json<McpServer>, AppError> {
    let visibility = req.visibility.unwrap_or_else(|| "public".to_string());

    let server = sqlx::query_as::<_, McpServer>(
        r#"UPDATE mcp_servers SET name=$2, transport=$3, namespace=$4, visibility=$5
           WHERE id=$1
           RETURNING id, name, transport, namespace, visibility, owner_org_id, created_at"#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.transport)
    .bind(&req.namespace)
    .bind(&visibility)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("MCP server '{}' not found", id)))?;

    Ok(Json(server))
}

pub async fn delete_mcp_server(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM mcp_servers WHERE id = $1")
        .bind(&id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("MCP server '{}' not found", id)));
    }

    Ok(Json(serde_json::json!({"deleted": true, "id": id})))
}

// ═══════════════════════════════════════════════════════════════
//  Skills CRUD
// ═══════════════════════════════════════════════════════════════

pub async fn create_skill(
    State(pool): State<PgPool>,
    Json(req): Json<CreateSkillRequest>,
) -> Result<Json<Skill>, AppError> {
    let id = req.name.to_lowercase().replace(' ', "-");

    let skill = sqlx::query_as::<_, Skill>(
        r#"INSERT INTO skills (id, scope, name, description)
           VALUES ($1, $2, $3, $4)
           RETURNING id, scope, owner_id, name, description"#,
    )
    .bind(&id)
    .bind(&req.scope)
    .bind(&req.name)
    .bind(&req.description)
    .fetch_one(&pool)
    .await?;

    Ok(Json(skill))
}

pub async fn list_skills(
    State(pool): State<PgPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Skill>>, AppError> {
    let per_page = q.per_page.unwrap_or(20);
    let offset = (q.page.unwrap_or(1) - 1) * per_page;

    let skills = sqlx::query_as::<_, Skill>(
        r#"SELECT id, scope, owner_id, name, description
           FROM skills ORDER BY name LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    Ok(Json(skills))
}

pub async fn get_skill(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Skill>, AppError> {
    let skill = sqlx::query_as::<_, Skill>(
        r#"SELECT id, scope, owner_id, name, description FROM skills WHERE id = $1"#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Skill '{}' not found", id)))?;

    Ok(Json(skill))
}

pub async fn update_skill(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(req): Json<CreateSkillRequest>,
) -> Result<Json<Skill>, AppError> {
    let skill = sqlx::query_as::<_, Skill>(
        r#"UPDATE skills SET scope=$2, name=$3, description=$4 WHERE id=$1
           RETURNING id, scope, owner_id, name, description"#,
    )
    .bind(&id)
    .bind(&req.scope)
    .bind(&req.name)
    .bind(&req.description)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Skill '{}' not found", id)))?;

    Ok(Json(skill))
}

pub async fn delete_skill(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM skills WHERE id = $1")
        .bind(&id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Skill '{}' not found", id)));
    }

    Ok(Json(serde_json::json!({"deleted": true, "id": id})))
}

pub async fn search_skills(
    State(pool): State<PgPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Skill>>, AppError> {
    let search_term = q.q.unwrap_or_default();
    let per_page = q.per_page.unwrap_or(20);

    let skills = sqlx::query_as::<_, Skill>(
        r#"SELECT id, scope, owner_id, name, description FROM skills
           WHERE to_tsvector('simple', name || ' ' || description) @@ plainto_tsquery('simple', $1)
           ORDER BY name LIMIT $2"#,
    )
    .bind(&search_term)
    .bind(per_page)
    .fetch_all(&pool)
    .await?;

    Ok(Json(skills))
}

// ═══════════════════════════════════════════════════════════════
//  Plugins CRUD
// ═══════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════
//  Installations
// ═══════════════════════════════════════════════════════════════

pub async fn create_installation(
    State(pool): State<PgPool>,
    Json(req): Json<CreateInstallationRequest>,
) -> Result<Json<Installation>, AppError> {
    let installation = sqlx::query_as::<_, Installation>(
        r#"INSERT INTO installations (entity_type, entity_id, version, scope, scope_ref)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, entity_type, entity_id, version, scope, scope_ref, installed_by, org_id, installed_at"#,
    )
    .bind(&req.entity_type).bind(&req.entity_id).bind(&req.version)
    .bind(&req.scope).bind(&req.scope_ref)
    .fetch_one(&pool).await?;

    Ok(Json(installation))
}

pub async fn list_installations(
    State(pool): State<PgPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Installation>>, AppError> {
    let per_page = q.per_page.unwrap_or(20);

    let installations = if let Some(scope) = &q.scope {
        sqlx::query_as::<_, Installation>(
            r#"SELECT id, entity_type, entity_id, version, scope, scope_ref, installed_by, org_id, installed_at
               FROM installations WHERE scope = $1 ORDER BY installed_at DESC LIMIT $2"#,
        )
        .bind(scope).bind(per_page)
        .fetch_all(&pool).await?
    } else {
        sqlx::query_as::<_, Installation>(
            r#"SELECT id, entity_type, entity_id, version, scope, scope_ref, installed_by, org_id, installed_at
               FROM installations ORDER BY installed_at DESC LIMIT $1"#,
        )
        .bind(per_page)
        .fetch_all(&pool).await?
    };

    Ok(Json(installations))
}

pub async fn get_installation(
    State(pool): State<PgPool>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Installation>, AppError> {
    let installation = sqlx::query_as::<_, Installation>(
        r#"SELECT id, entity_type, entity_id, version, scope, scope_ref, installed_by, org_id, installed_at
           FROM installations WHERE id = $1"#,
    )
    .bind(id).fetch_optional(&pool).await?
    .ok_or_else(|| AppError::NotFound("Installation not found".to_string()))?;

    Ok(Json(installation))
}

pub async fn delete_installation(
    State(pool): State<PgPool>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM installations WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Installation not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({"deleted": true, "id": id.to_string()}),
    ))
}
