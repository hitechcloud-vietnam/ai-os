use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket},
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{get, post},
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  AG-UI Protocol — Agent-to-UI Event Streaming
// ═══════════════════════════════════════════════════════════════

/// AG-UI event types as defined in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum AgUiEvent {
    #[serde(rename = "agent.session.start")]
    SessionStart {
        session_id: String,
        agent_id: String,
    },
    #[serde(rename = "agent.text.delta")]
    TextDelta { session_id: String, delta: String },
    #[serde(rename = "agent.thought.delta")]
    ThoughtDelta { session_id: String, delta: String },
    #[serde(rename = "agent.tool.calling")]
    ToolCalling {
        session_id: String,
        tool: String,
        parameters: serde_json::Value,
    },
    #[serde(rename = "agent.tool.executed")]
    ToolExecuted {
        session_id: String,
        tool: String,
        result: serde_json::Value,
        duration_ms: u64,
    },
    #[serde(rename = "agent.state.changed")]
    StateChanged {
        session_id: String,
        state: AgentState,
    },
    #[serde(rename = "agent.hitl.request")]
    HitlRequest {
        session_id: String,
        request_id: String,
        severity: String,
        action: HitlAction,
    },
    #[serde(rename = "agent.session.end")]
    SessionEnd { session_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Thinking,
    Executing,
    Interrupted,
    WaitingForInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlAction {
    pub tool: String,
    pub parameters: serde_json::Value,
    pub explanation: String,
}

/// Client-to-server events
#[derive(Debug, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum ClientEvent {
    #[serde(rename = "client.hitl.response")]
    HitlResponse {
        request_id: String,
        approved: bool,
        reason: Option<String>,
    },
    #[serde(rename = "client.interrupt")]
    Interrupt { session_id: String },
    #[serde(rename = "client.message")]
    Message { session_id: String, content: String },
}

/// Session state
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub state: AgentState,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Shared session store
pub type SessionStore = Arc<RwLock<HashMap<String, Session>>>;

// ═══════════════════════════════════════════════════════════════
//  SSE Event Stream
// ═══════════════════════════════════════════════════════════════

async fn sse_stream(
    State(store): State<SessionStore>,
    Path(session_id): Path<String>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        let max_iterations = 7200; // 1 hour max

        for _ in 0..max_iterations {
            interval.tick().await;

            let sessions = store.read().await;
            if let Some(session) = sessions.get(&session_id) {
                let event = AgUiEvent::StateChanged {
                    session_id: session_id.clone(),
                    state: session.state.clone(),
                };

                let data = serde_json::to_string(&event).unwrap_or_default();
                yield Ok(Event::default().event("agent.state.changed").data(data));

                // Check if session ended
                if matches!(session.state, AgentState::Idle) {
                    // Keep alive but don't flood
                }
            } else {
                yield Ok(Event::default().event("agent.session.end").data(format!("{{\"session_id\":\"{}\"}}", session_id)));
                break;
            }
        }
    };

    Ok(Sse::new(stream))
}

// ═══════════════════════════════════════════════════════════════
//  WebSocket Handler
// ═══════════════════════════════════════════════════════════════

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(store): State<SessionStore>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, store))
}

async fn handle_socket(socket: WebSocket, store: SessionStore) {
    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let welcome = serde_json::json!({
        "event": "agent.session.start",
        "data": {
            "session_id": Uuid::new_v4().to_string(),
            "message": "Connected to HiTechCloud AG-UI"
        }
    });
    let _ = sender.send(Message::Text(welcome.to_string().into())).await;

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                    match event {
                        ClientEvent::HitlResponse {
                            request_id,
                            approved,
                            reason,
                        } => {
                            tracing::info!(request_id = %request_id, approved = approved, reason = ?reason, "HITL response received");
                            let response = serde_json::json!({
                                "event": "hitl.processed",
                                "request_id": request_id,
                                "approved": approved
                            });
                            let _ = sender
                                .send(Message::Text(response.to_string().into()))
                                .await;
                        }
                        ClientEvent::Interrupt { session_id } => {
                            tracing::info!(session_id = %session_id, "Client interrupted");
                            let mut sessions = store.write().await;
                            if let Some(session) = sessions.get_mut(&session_id) {
                                session.state = AgentState::Interrupted;
                            }
                        }
                        ClientEvent::Message {
                            session_id,
                            content,
                        } => {
                            tracing::info!(session_id = %session_id, content = %content, "Client message");
                            // Echo back for now
                            let response = serde_json::json!({
                                "event": "agent.text.delta",
                                "data": { "session_id": session_id, "delta": format!("Received: {}", content) }
                            });
                            let _ = sender
                                .send(Message::Text(response.to_string().into()))
                                .await;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  Session Management API
// ═══════════════════════════════════════════════════════════════

async fn create_session(
    State(store): State<SessionStore>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let session_id = Uuid::new_v4().to_string();
    let agent_id = req
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let session = Session {
        id: session_id.clone(),
        agent_id: agent_id.clone(),
        state: AgentState::Idle,
        created_at: chrono::Utc::now(),
    };

    store.write().await.insert(session_id.clone(), session);

    Json(serde_json::json!({
        "session_id": session_id,
        "agent_id": agent_id,
        "state": "idle"
    }))
}

async fn list_sessions(State(store): State<SessionStore>) -> Json<Vec<serde_json::Value>> {
    let sessions = store.read().await;
    let list: Vec<_> = sessions
        .values()
        .map(|s| {
            serde_json::json!({
                "session_id": s.id,
                "agent_id": s.agent_id,
                "state": format!("{:?}", s.state).to_lowercase(),
                "created_at": s.created_at.to_rfc3339()
            })
        })
        .collect();
    Json(list)
}

async fn get_session(
    State(store): State<SessionStore>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sessions = store.read().await;
    match sessions.get(&session_id) {
        Some(s) => Ok(Json(serde_json::json!({
            "session_id": s.id,
            "agent_id": s.agent_id,
            "state": format!("{:?}", s.state).to_lowercase(),
            "created_at": s.created_at.to_rfc3339()
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn emit_event(
    State(store): State<SessionStore>,
    Path(session_id): Path<String>,
    Json(event): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut sessions = store.write().await;
    match sessions.get_mut(&session_id) {
        Some(session) => {
            // Update state if provided
            if let Some(state_str) = event.get("state").and_then(|v| v.as_str()) {
                session.state = match state_str {
                    "thinking" => AgentState::Thinking,
                    "executing" => AgentState::Executing,
                    "interrupted" => AgentState::Interrupted,
                    "waiting_for_input" => AgentState::WaitingForInput,
                    _ => AgentState::Idle,
                };
            }
            Ok(Json(
                serde_json::json!({"status": "emitted", "session_id": session_id}),
            ))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ═══════════════════════════════════════════════════════════════
//  Health
// ═══════════════════════════════════════════════════════════════

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "hitechcloud-ag-ui"
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

    tracing::info!("Starting HiTechCloud AG-UI v0.1.0");

    let port = std::env::var("AGUI_PORT")
        .unwrap_or_else(|_| "8086".to_string())
        .parse::<u16>()
        .unwrap_or(8086);

    let store: SessionStore = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/health", get(health))
        // Session management
        .route("/agui/v1/sessions", post(create_session).get(list_sessions))
        .route("/agui/v1/sessions/{session_id}", get(get_session))
        .route("/agui/v1/sessions/{session_id}/emit", post(emit_event))
        // SSE event stream
        .route("/agui/v1/events/{session_id}", get(sse_stream))
        // WebSocket
        .route("/agui/v1/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(store);

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("AG-UI listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
