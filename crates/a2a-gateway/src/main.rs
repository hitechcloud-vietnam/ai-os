use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::convert::Infallible;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  A2A Models — Agent-to-Agent Protocol v1.0
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentCard {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub description: Option<String>,
    pub publisher: Option<serde_json::Value>,
    pub capabilities: serde_json::Value,
    pub endpoints: Option<serde_json::Value>,
    pub auth_type: Option<String>,
    pub auth_scopes: Option<Vec<String>>,
    pub status: String,
    pub org_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub agent_id: String,
    pub name: String,
    pub description: Option<String>,
    pub publisher: Option<serde_json::Value>,
    pub capabilities: serde_json::Value,
    pub endpoints: Option<serde_json::Value>,
    pub auth_type: Option<String>,
    pub auth_scopes: Option<Vec<String>>,
    pub org_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct A2aTask {
    pub id: Uuid,
    pub task_id: String,
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub action: String,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub status: String,
    pub progress_pct: i32,
    pub error_message: Option<String>,
    pub timeout_ms: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DispatchTaskRequest {
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub action: String,
    pub input: serde_json::Value,
    pub timeout_ms: Option<i32>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TaskFilter {
    pub status: Option<String>,
    pub agent_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct A2aTaskEvent {
    pub event: String,
    pub task_id: String,
    pub status: String,
    pub progress_pct: i32,
    pub data: Option<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════
//  Agent Card CRUD
// ═══════════════════════════════════════════════════════════════

async fn register_agent(
    State(pool): State<PgPool>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentCard>, StatusCode> {
    let agent = sqlx::query_as::<_, AgentCard>(
        "INSERT INTO a2a_agent_cards (agent_id, name, description, publisher, capabilities, endpoints, auth_type, auth_scopes, org_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, agent_id, name, description, publisher, capabilities, endpoints, auth_type, auth_scopes, status, org_id, created_at, updated_at"
    )
    .bind(&req.agent_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.publisher)
    .bind(&req.capabilities)
    .bind(&req.endpoints)
    .bind(&req.auth_type)
    .bind(&req.auth_scopes)
    .bind(req.org_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to register agent: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(agent))
}

async fn list_agents(State(pool): State<PgPool>) -> Result<Json<Vec<AgentCard>>, StatusCode> {
    let agents = sqlx::query_as::<_, AgentCard>(
        "SELECT id, agent_id, name, description, publisher, capabilities, endpoints, auth_type, auth_scopes, status, org_id, created_at, updated_at FROM a2a_agent_cards WHERE status = 'active' ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(agents))
}

async fn get_agent(
    State(pool): State<PgPool>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentCard>, StatusCode> {
    let agent = sqlx::query_as::<_, AgentCard>(
        "SELECT id, agent_id, name, description, publisher, capabilities, endpoints, auth_type, auth_scopes, status, org_id, created_at, updated_at FROM a2a_agent_cards WHERE agent_id = $1"
    )
    .bind(&agent_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match agent {
        Some(a) => Ok(Json(a)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn deactivate_agent(
    State(pool): State<PgPool>,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "UPDATE a2a_agent_cards SET status = 'inactive', updated_at = NOW() WHERE agent_id = $1",
    )
    .bind(&agent_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// ═══════════════════════════════════════════════════════════════
//  Task Dispatch & Lifecycle
// ═══════════════════════════════════════════════════════════════

async fn dispatch_task(
    State(pool): State<PgPool>,
    Json(req): Json<DispatchTaskRequest>,
) -> Result<Json<A2aTask>, StatusCode> {
    let task_id = format!(
        "task_{}",
        Uuid::new_v4().to_string().replace('-', "")[..12].to_string()
    );

    // Verify target agent exists and is active
    let target =
        sqlx::query_scalar::<_, String>("SELECT status FROM a2a_agent_cards WHERE agent_id = $1")
            .bind(&req.target_agent_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match target {
        Some(status) if status == "active" => {}
        Some(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
        None => return Err(StatusCode::NOT_FOUND),
    }

    let task = sqlx::query_as::<_, A2aTask>(
        "INSERT INTO a2a_tasks (task_id, source_agent_id, target_agent_id, action, input_data, timeout_ms)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at"
    )
    .bind(&task_id)
    .bind(&req.source_agent_id)
    .bind(&req.target_agent_id)
    .bind(&req.action)
    .bind(&req.input)
    .bind(req.timeout_ms.unwrap_or(30000))
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to dispatch task: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        task_id = %task_id,
        source = %req.source_agent_id,
        target = %req.target_agent_id,
        action = %req.action,
        "A2A task dispatched"
    );

    Ok(Json(task))
}

async fn get_task(
    State(pool): State<PgPool>,
    Path(task_id): Path<String>,
) -> Result<Json<A2aTask>, StatusCode> {
    let task = sqlx::query_as::<_, A2aTask>(
        "SELECT id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at FROM a2a_tasks WHERE task_id = $1"
    )
    .bind(&task_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match task {
        Some(t) => Ok(Json(t)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn list_tasks(
    State(pool): State<PgPool>,
    Query(filter): Query<TaskFilter>,
) -> Result<Json<Vec<A2aTask>>, StatusCode> {
    let limit = filter.limit.unwrap_or(20).min(100);

    let tasks = if let Some(ref status) = filter.status {
        sqlx::query_as::<_, A2aTask>(
            "SELECT id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at FROM a2a_tasks WHERE status = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(status)
        .bind(limit)
        .fetch_all(&pool)
        .await
    } else if let Some(ref agent_id) = filter.agent_id {
        sqlx::query_as::<_, A2aTask>(
            "SELECT id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at FROM a2a_tasks WHERE source_agent_id = $1 OR target_agent_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, A2aTask>(
            "SELECT id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at FROM a2a_tasks ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&pool)
        .await
    };

    tasks
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn update_task_status(
    State(pool): State<PgPool>,
    Path(task_id): Path<String>,
    Json(update): Json<serde_json::Value>,
) -> Result<Json<A2aTask>, StatusCode> {
    let status = update
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("in_progress");
    let progress = update
        .get("progress_pct")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let output = update.get("output").cloned();
    let error = update.get("error_message").and_then(|v| v.as_str());

    let task = sqlx::query_as::<_, A2aTask>(
        "UPDATE a2a_tasks SET status = $1, progress_pct = $2, output_data = $3, error_message = $4, updated_at = NOW()
         WHERE task_id = $5
         RETURNING id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at"
    )
    .bind(status)
    .bind(progress)
    .bind(output)
    .bind(error)
    .bind(&task_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match task {
        Some(t) => Ok(Json(t)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ═══════════════════════════════════════════════════════════════
//  SSE Task Stream
// ═══════════════════════════════════════════════════════════════

async fn task_stream(
    State(pool): State<PgPool>,
    Path(task_id): Path<String>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let stream = async_stream::stream! {
        let mut last_status = String::new();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        let max_iterations = 300; // 5 minutes max

        for _ in 0..max_iterations {
            interval.tick().await;

            let task = sqlx::query_as::<_, A2aTask>(
                "SELECT id, task_id, source_agent_id, target_agent_id, action, input_data, output_data, status, progress_pct, error_message, timeout_ms, created_at, updated_at FROM a2a_tasks WHERE task_id = $1"
            )
            .bind(&task_id)
            .fetch_optional(&pool)
            .await;

            if let Ok(Some(t)) = task {
                if t.status != last_status || t.progress_pct > 0 {
                    let event_type = match t.status.as_str() {
                        "completed" => "a2a.task.completed",
                        "failed" => "a2a.task.failed",
                        "requires_input" => "a2a.task.requires_input",
                        _ => "a2a.task.progress",
                    };

                    let data = serde_json::json!({
                        "task_id": t.task_id,
                        "status": t.status,
                        "progress_pct": t.progress_pct,
                        "output": t.output_data,
                        "error": t.error_message,
                    });

                    yield Ok(Event::default()
                        .event(event_type)
                        .data(serde_json::to_string(&data).unwrap_or_default()));

                    last_status = t.status.clone();

                    if t.status == "completed" || t.status == "failed" {
                        break;
                    }
                }
            } else {
                yield Ok(Event::default()
                    .event("a2a.task.error")
                    .data("{\"error\": \"task not found\"}"));
                break;
            }
        }
    };

    Ok(Sse::new(stream))
}

// ═══════════════════════════════════════════════════════════════
//  A2A Discovery
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct DiscoverQuery {
    capability: Option<String>,
}

async fn discover_agents(
    State(pool): State<PgPool>,
    Query(query): Query<DiscoverQuery>,
) -> Result<Json<Vec<AgentCard>>, StatusCode> {
    let agents = if let Some(cap) = &query.capability {
        sqlx::query_as::<_, AgentCard>(
            "SELECT id, agent_id, name, description, publisher, capabilities, endpoints, auth_type, auth_scopes, status, org_id, created_at, updated_at FROM a2a_agent_cards WHERE status = 'active' AND capabilities::text ILIKE $1 ORDER BY name"
        )
        .bind(format!("%{}%", cap))
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, AgentCard>(
            "SELECT id, agent_id, name, description, publisher, capabilities, endpoints, auth_type, auth_scopes, status, org_id, created_at, updated_at FROM a2a_agent_cards WHERE status = 'active' ORDER BY name"
        )
        .fetch_all(&pool)
        .await
    };

    agents
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ═══════════════════════════════════════════════════════════════
//  Health
// ═══════════════════════════════════════════════════════════════

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "hitechcloud-a2a-gateway"
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

    tracing::info!("Starting HiTechCloud A2A Gateway v0.1.0");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://hitechcloud:hitechcloud_dev_2026@127.0.0.1:5432/hitechcloud".to_string()
    });

    let port = std::env::var("A2A_PORT")
        .unwrap_or_else(|_| "8084".to_string())
        .parse::<u16>()
        .unwrap_or(8084);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    tracing::info!("Connected to PostgreSQL");

    // Create A2A tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS a2a_agent_cards (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            agent_id VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            publisher JSONB,
            capabilities JSONB NOT NULL DEFAULT '[]',
            endpoints JSONB,
            auth_type VARCHAR(64),
            auth_scopes TEXT[],
            status VARCHAR(32) NOT NULL DEFAULT 'active',
            org_id UUID NOT NULL REFERENCES organizations(id),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS a2a_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            task_id VARCHAR(64) UNIQUE NOT NULL,
            source_agent_id VARCHAR(255) NOT NULL,
            target_agent_id VARCHAR(255) NOT NULL,
            action VARCHAR(255) NOT NULL,
            input_data JSONB NOT NULL DEFAULT '{}',
            output_data JSONB,
            status VARCHAR(32) NOT NULL DEFAULT 'created',
            progress_pct INT NOT NULL DEFAULT 0,
            error_message TEXT,
            timeout_ms INT NOT NULL DEFAULT 30000,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await?;

    tracing::info!("A2A tables ready");

    let app = Router::new()
        .route("/health", get(health))
        // Agent Card CRUD
        .route("/a2a/v1/agents", post(register_agent).get(list_agents))
        .route(
            "/a2a/v1/agents/{agent_id}",
            get(get_agent).delete(deactivate_agent),
        )
        // Agent Discovery
        .route("/a2a/v1/discover", get(discover_agents))
        // Task Dispatch & Lifecycle
        .route("/a2a/v1/tasks", post(dispatch_task).get(list_tasks))
        .route(
            "/a2a/v1/tasks/{task_id}",
            get(get_task).put(update_task_status),
        )
        // SSE Task Stream
        .route("/a2a/v1/stream/{task_id}", get(task_stream))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("A2A Gateway listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
