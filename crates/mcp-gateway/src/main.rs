use axum::{
    Json, Router,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream::Stream;
use hitechcloud_common::{
    models::HealthResponse,
    rpc::{JsonRpcRequest, JsonRpcResponse},
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::convert::Infallible;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod mcp_handler;
mod routing;
mod sse_handler;

#[derive(Clone)]
pub struct McpState {
    pub pool: sqlx::PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub http_client: reqwest::Client,
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

    tracing::info!("Starting HiTechCloud MCP Gateway v0.1.0");

    let cfg = config::McpConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(cfg.database_max_connections)
        .connect(&cfg.database_url)
        .await?;
    tracing::info!("Connected to PostgreSQL");

    let redis_client = redis::Client::open(cfg.redis_url.clone())?;
    let redis = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("Connected to Redis");

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let state = McpState {
        pool,
        redis,
        http_client,
    };

    let app = Router::new()
        // Health
        .route("/health", get(health))
        // MCP JSON-RPC 2.0 endpoint
        .route("/mcp/v1", post(mcp_jsonrpc_handler))
        // MCP SSE endpoint
        .route("/mcp/v1/sse", get(mcp_sse_handler))
        // Tool execution (OpenAI-compatible)
        .route("/tool-execute", post(tool_execute_handler))
        // List available tools
        .route("/tools", get(list_tools_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let bind_addr = format!("0.0.0.0:{}", cfg.port);
    tracing::info!("MCP Gateway listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "hitechcloud-mcp-gateway".to_string(),
    })
}

/// MCP JSON-RPC 2.0 handler - routes to appropriate MCP servers
async fn mcp_jsonrpc_handler(
    State(state): State<McpState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    Json(mcp_handler::handle_mcp_request(state, request).await)
}

/// SSE endpoint for MCP streaming
async fn mcp_sse_handler(
    State(state): State<McpState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(sse_handler::create_sse_stream(state)).keep_alive(KeepAlive::default())
}

/// OpenAI-compatible tool execution
async fn tool_execute_handler(
    State(state): State<McpState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let tool_name = payload
        .get("tool")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let _args = payload.get("arguments").cloned().unwrap_or(json!({}));

    // Route tool call to appropriate MCP server
    match routing::route_tool_call(&state, tool_name, &_args).await {
        Ok(result) => Json(json!({
            "output": result,
            "status": "success"
        })),
        Err(e) => Json(json!({
            "error": e.to_string(),
            "status": "error"
        })),
    }
}

/// List all available tools across all registered MCP servers
async fn list_tools_handler(State(state): State<McpState>) -> Json<serde_json::Value> {
    match routing::list_all_tools(&state).await {
        Ok(tools) => Json(json!({
            "tools": tools,
            "count": tools.len()
        })),
        Err(e) => Json(json!({
            "error": e.to_string(),
            "tools": [],
            "count": 0
        })),
    }
}
