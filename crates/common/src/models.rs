use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Organizations ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

// ── Users ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

// ── API Keys ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub key_hash: String,
    pub scope: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── MCP Servers ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub namespace: String,
    pub visibility: String,
    pub owner_org_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpServerVersion {
    pub mcp_server_id: String,
    pub version: String,
    pub manifest: serde_json::Value,
    pub package_hash: String,
    pub signature: Option<String>,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
}

// ── Skills ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    pub id: String,
    pub scope: String,
    pub owner_id: Option<Uuid>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillVersion {
    pub skill_id: String,
    pub version: String,
    pub content_ref: String,
    pub status: String,
}

// ── Plugins ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plugin {
    pub id: String,
    pub owner_org_id: Option<Uuid>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PluginVersion {
    pub plugin_id: String,
    pub version: String,
    pub manifest: serde_json::Value,
    pub package_hash: String,
    pub signature: Option<String>,
    pub status: String,
}

// ── Installations ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Installation {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub version: String,
    pub scope: String,
    pub scope_ref: Option<String>,
    pub installed_by: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub installed_at: DateTime<Utc>,
}

// ── Audit Logs ──
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub org_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub result_status: String,
    pub request_meta: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ── API Request/Response types ──
#[derive(Debug, Deserialize)]
pub struct CreateMcpServerRequest {
    pub name: String,
    pub transport: String,
    pub namespace: String,
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub name: String,
    pub scope: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePluginRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInstallationRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub version: String,
    pub scope: String,
    pub scope_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub scope: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
}
