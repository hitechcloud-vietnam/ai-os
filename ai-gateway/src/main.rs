use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use hitechcloud_common::models::HealthResponse;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod routes;

#[derive(Clone)]
struct AuthState {
    pool: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting HiTechCloud AI Gateway v0.1.0");

    let cfg = config::AppConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(cfg.database_max_connections)
        .connect(&cfg.database_url)
        .await?;
    tracing::info!("Connected to PostgreSQL");

    let redis_client = redis::Client::open(cfg.redis_url.clone())?;
    let _redis = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("Connected to Redis");

    let auth_state = AuthState { pool: pool.clone() };

    // ═══════════════════════════════════════════════════════════════
    //  Build unified Router<PgPool> — all routes with explicit prefixes
    // ═══════════════════════════════════════════════════════════════
    let app = Router::<PgPool>::new()
        // Health (no auth required — handled in middleware)
        .route("/health", get(health))
        .route("/v1/health", get(health))
        // ── Registry: MCP Servers (full CRUD) ──
        .route("/v1/registry/mcp-servers", post(hitechcloud_registry::create_mcp_server).get(hitechcloud_registry::list_mcp_servers))
        .route("/v1/registry/mcp-servers/{id}", get(hitechcloud_registry::get_mcp_server).put(hitechcloud_registry::update_mcp_server).delete(hitechcloud_registry::delete_mcp_server))
        // ── Registry: Skills (full CRUD) ──
        .route("/v1/registry/skills", post(hitechcloud_registry::create_skill).get(hitechcloud_registry::list_skills))
        .route("/v1/registry/skills/{id}", get(hitechcloud_registry::get_skill).put(hitechcloud_registry::update_skill).delete(hitechcloud_registry::delete_skill))
        .route("/v1/registry/skills/search", get(hitechcloud_registry::search_skills))
        // ── Registry: Plugins (full CRUD) ──
        .route("/v1/registry/plugins", post(hitechcloud_registry::create_plugin).get(hitechcloud_registry::list_plugins))
        .route("/v1/registry/plugins/{id}", get(hitechcloud_registry::get_plugin).put(hitechcloud_registry::update_plugin).delete(hitechcloud_registry::delete_plugin))
        .route("/v1/registry/plugins/{id}/versions", get(hitechcloud_registry::get_plugin_versions))
        // ── Registry: Installations ──
        .route("/v1/registry/installations", post(hitechcloud_registry::create_installation).get(hitechcloud_registry::list_installations))
        .route("/v1/registry/installations/{id}", get(hitechcloud_registry::get_installation).delete(hitechcloud_registry::delete_installation))
        // ── Skills Service ──
        .route("/v1/skills", get(hitechcloud_skills_service::list_skills).post(hitechcloud_skills_service::create_skill))
        .route("/v1/skills/{id}", get(hitechcloud_skills_service::get_skill).put(hitechcloud_skills_service::update_skill).delete(hitechcloud_skills_service::delete_skill))
        .route("/v1/skills/{id}/versions", get(hitechcloud_skills_service::get_skill_versions))
        // ── Plugin Service ──
        .route("/v1/plugins", get(hitechcloud_plugin_service::list_plugins).post(hitechcloud_plugin_service::create_plugin))
        .route("/v1/plugins/{id}", get(hitechcloud_plugin_service::get_plugin).put(hitechcloud_plugin_service::update_plugin).delete(hitechcloud_plugin_service::delete_plugin))
        .route("/v1/plugins/{id}/versions", get(hitechcloud_plugin_service::get_plugin_versions))
        // ── Marketplace ──
        .route("/v1/marketplace/search", get(routes::marketplace::search))
        .route("/v1/marketplace/featured", get(routes::marketplace::featured))
        // ── Admin ──
        .route("/v1/admin/orgs", get(routes::admin::list_orgs))
        .route("/v1/admin/orgs/{org_id}/members", get(routes::admin::get_members))
        .route("/v1/admin/orgs/{org_id}/audit-logs", get(routes::admin::get_audit_logs))
        // ── Middleware ──
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let bind_addr = format!("0.0.0.0:{}", cfg.port);
    tracing::info!("AI Gateway listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(_pool): State<PgPool>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "hitechcloud-ai-gateway".to_string(),
    })
}

async fn auth_middleware(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    if path == "/health" || path == "/v1/health" {
        return Ok(next.run(request).await);
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let key_hash = hitechcloud_signing::sha256_checksum(token.as_bytes());
            let result = sqlx::query_scalar::<_, uuid::Uuid>(
                "SELECT id FROM api_keys WHERE key_hash = $1 AND (expires_at IS NULL OR expires_at > now())",
            )
            .bind(&key_hash)
            .fetch_optional(&state.pool)
            .await;

            if result.ok().flatten().is_some() {
                return Ok(next.run(request).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
