use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  Plugin Gateway — Lifecycle orchestration
// ═══════════════════════════════════════════════════════════════

/// Plugin manifest (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub mcp: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub compatible_agents: Vec<String>,
}

/// Installation scope
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    User,
    Project,
    Org,
}

impl std::fmt::Display for InstallScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallScope::User => write!(f, "user"),
            InstallScope::Project => write!(f, "project"),
            InstallScope::Org => write!(f, "org"),
        }
    }
}

/// Plugin lifecycle state
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    Draft,
    Submitted,
    Reviewed,
    Signed,
    Published,
    Deprecated,
    Yanked,
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginState::Draft => write!(f, "draft"),
            PluginState::Submitted => write!(f, "submitted"),
            PluginState::Reviewed => write!(f, "reviewed"),
            PluginState::Signed => write!(f, "signed"),
            PluginState::Published => write!(f, "published"),
            PluginState::Deprecated => write!(f, "deprecated"),
            PluginState::Yanked => write!(f, "yanked"),
        }
    }
}

/// Plugin record in database
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginRecord {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub state: String,
    pub manifest: serde_json::Value,
    pub signature: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Plugin installation record
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginInstallation {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub org_id: Uuid,
    pub user_id: Option<Uuid>,
    pub scope: String,
    pub installed_version: String,
    pub permissions: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

// ═══════════════════════════════════════════════════════════════
//  Handlers — Plugin CRUD
// ═══════════════════════════════════════════════════════════════

async fn list_plugins(State(state): State<AppState>) -> Json<Vec<PluginRecord>> {
    match sqlx::query_as::<_, PluginRecord>(
        "SELECT id, name, version, description, author, state, manifest, signature, created_at, updated_at FROM plugins ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(list) => Json(list),
        Err(e) => {
            tracing::error!("list_plugins: {}", e);
            Json(vec![])
        }
    }
}

async fn get_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PluginRecord>, StatusCode> {
    sqlx::query_as::<_, PluginRecord>(
        "SELECT id, name, version, description, author, state, manifest, signature, created_at, updated_at FROM plugins WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(Json)
    .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct CreatePlugin {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    manifest: serde_json::Value,
}

async fn create_plugin(
    State(state): State<AppState>,
    Json(req): Json<CreatePlugin>,
) -> Result<(StatusCode, Json<PluginRecord>), StatusCode> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO plugins (id, name, version, description, author, state, manifest, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.version)
    .bind(&req.description)
    .bind(&req.author)
    .bind("draft")
    .bind(&req.manifest)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record = PluginRecord {
        id,
        name: req.name,
        version: req.version,
        description: req.description,
        author: req.author,
        state: "draft".to_string(),
        manifest: req.manifest,
        signature: None,
        created_at: now,
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(record)))
}

// ═══════════════════════════════════════════════════════════════
//  Handlers — Plugin Lifecycle
// ═══════════════════════════════════════════════════════════════

async fn submit_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query("UPDATE plugins SET state = 'submitted', updated_at = NOW() WHERE id = $1 AND state = 'draft'")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({"status": "submitted", "plugin_id": id})))
}

async fn review_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let approved = req.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
    let new_state = if approved { "reviewed" } else { "draft" };

    let result = sqlx::query("UPDATE plugins SET state = $1, updated_at = NOW() WHERE id = $2 AND state = 'submitted'")
        .bind(new_state)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({"status": new_state, "plugin_id": id})))
}

async fn sign_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Generate a mock signature (in production, use Ed25519)
    let signature = format!("ed25519:{}", Uuid::new_v4().to_string().replace('-', ""));

    let result = sqlx::query("UPDATE plugins SET state = 'signed', signature = $1, updated_at = NOW() WHERE id = $2 AND state = 'reviewed'")
        .bind(&signature)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({"status": "signed", "plugin_id": id, "signature": signature})))
}

async fn publish_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query("UPDATE plugins SET state = 'published', updated_at = NOW() WHERE id = $1 AND state = 'signed'")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({"status": "published", "plugin_id": id})))
}

async fn deprecate_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query("UPDATE plugins SET state = 'deprecated', updated_at = NOW() WHERE id = $1 AND state = 'published'")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({"status": "deprecated", "plugin_id": id})))
}

// ═══════════════════════════════════════════════════════════════
//  Handlers — Installation
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
#[allow(dead_code)]
struct InstallRequest {
    org_id: Uuid,
    user_id: Option<Uuid>,
    scope: Option<String>,
}

async fn install_plugin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<InstallRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    // Check plugin exists and is published
    let plugin = sqlx::query_as::<_, PluginRecord>(
        "SELECT id, name, version, description, author, state, manifest, signature, created_at, updated_at FROM plugins WHERE id = $1 AND state = 'published'"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let scope = req.scope.unwrap_or_else(|| "org".to_string());
    let install_id = Uuid::new_v4();

    // Parse manifest for permissions
    let permissions = plugin.manifest.get("permissions")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));

    sqlx::query(
        "INSERT INTO installations (id, plugin_id, org_id, status, installed_at) VALUES ($1, $2, $3, 'active', NOW())"
    )
    .bind(install_id)
    .bind(id)
    .bind(req.org_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(
        plugin = %plugin.name,
        version = %plugin.version,
        org_id = %req.org_id,
        scope = %scope,
        "Plugin installed"
    );

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "installation_id": install_id,
        "plugin_id": id,
        "plugin_name": plugin.name,
        "version": plugin.version,
        "scope": scope,
        "permissions": permissions,
        "status": "active"
    }))))
}

async fn uninstall_plugin(
    State(state): State<AppState>,
    Path((plugin_id, org_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query("UPDATE installations SET status = 'removed' WHERE plugin_id = $1 AND org_id = $2 AND status = 'active'")
        .bind(plugin_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({
        "status": "uninstalled",
        "plugin_id": plugin_id,
        "org_id": org_id
    })))
}

async fn list_installations(State(state): State<AppState>) -> Json<Vec<PluginInstallation>> {
    match sqlx::query_as::<_, PluginInstallation>(
        "SELECT id, plugin_id, org_id, NULL as user_id, 'org' as scope, '1.0.0' as installed_version, '[]'::jsonb as permissions, installed_at as created_at FROM installations WHERE status = 'active' ORDER BY installed_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(list) => Json(list),
        Err(e) => {
            tracing::error!("list_installations: {}", e);
            Json(vec![])
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  Health
// ═══════════════════════════════════════════════════════════════

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "hitechcloud-plugin-gateway"
    }))
}

// ═══════════════════════════════════════════════════════════════
//  Main
// ═══════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting HiTechCloud Plugin Gateway v0.1.0");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://hitechcloud:hitechcloud_dev_2026@localhost:5432/hitechcloud".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(health))
        // Plugin CRUD
        .route("/pgw/v1/plugins", get(list_plugins).post(create_plugin))
        .route("/pgw/v1/plugins/{id}", get(get_plugin))
        // Plugin lifecycle
        .route("/pgw/v1/plugins/{id}/submit", post(submit_plugin))
        .route("/pgw/v1/plugins/{id}/review", post(review_plugin))
        .route("/pgw/v1/plugins/{id}/sign", post(sign_plugin))
        .route("/pgw/v1/plugins/{id}/publish", post(publish_plugin))
        .route("/pgw/v1/plugins/{id}/deprecate", post(deprecate_plugin))
        // Installations
        .route("/pgw/v1/install", post(install_plugin))
        .route("/pgw/v1/uninstall/{plugin_id}/{org_id}", delete(uninstall_plugin))
        .route("/pgw/v1/installations", get(list_installations))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PLUGIN_GATEWAY_PORT")
        .unwrap_or_else(|_| "8087".to_string())
        .parse::<u16>()
        .unwrap_or(8087);

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("Plugin Gateway listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}