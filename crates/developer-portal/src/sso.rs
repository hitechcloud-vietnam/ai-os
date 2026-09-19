use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  SSO/SCIM — Enterprise Single Sign-On
// ═══════════════════════════════════════════════════════════════

/// SSO Configuration for an organization
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SsoConfig {
    pub id: Uuid,
    pub org_id: Uuid,
    pub provider: String,
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub metadata_url: Option<String>,
    pub enabled: bool,
    pub enforce_sso: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSsoRequest {
    pub org_id: Uuid,
    pub provider: String,
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub metadata_url: Option<String>,
    pub enforce_sso: Option<bool>,
}

/// SCIM User provisioning record
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScimUser {
    pub id: Uuid,
    pub org_id: Uuid,
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub synced_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ScimUserRequest {
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub active: Option<bool>,
    pub roles: Option<Vec<String>>,
}

/// SCIM Group
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScimGroup {
    pub id: Uuid,
    pub org_id: Uuid,
    pub external_id: String,
    pub display_name: String,
    pub members: Vec<String>,
    pub synced_at: chrono::DateTime<chrono::Utc>,
}

// ═══════════════════════════════════════════════════════════════
//  SSO Configuration CRUD
// ═══════════════════════════════════════════════════════════════

pub async fn create_sso_config(
    State(pool): State<PgPool>,
    Json(req): Json<CreateSsoRequest>,
) -> Result<Json<SsoConfig>, StatusCode> {
    let config = sqlx::query_as::<_, SsoConfig>(
        "INSERT INTO sso_configs (org_id, provider, issuer_url, client_id, metadata_url, enforce_sso)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, org_id, provider, issuer_url, client_id, metadata_url, enabled, enforce_sso, created_at, updated_at"
    )
    .bind(req.org_id)
    .bind(&req.provider)
    .bind(&req.issuer_url)
    .bind(&req.client_id)
    .bind(&req.metadata_url)
    .bind(req.enforce_sso.unwrap_or(false))
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create SSO config: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(org_id = %req.org_id, provider = %req.provider, "SSO config created");
    Ok(Json(config))
}

pub async fn get_sso_config(
    State(pool): State<PgPool>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<SsoConfig>, StatusCode> {
    let config = sqlx::query_as::<_, SsoConfig>(
        "SELECT id, org_id, provider, issuer_url, client_id, metadata_url, enabled, enforce_sso, created_at, updated_at FROM sso_configs WHERE org_id = $1"
    )
    .bind(org_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match config {
        Some(c) => Ok(Json(c)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_sso_config(
    State(pool): State<PgPool>,
    Path(org_id): Path<Uuid>,
    Json(update): Json<serde_json::Value>,
) -> Result<Json<SsoConfig>, StatusCode> {
    let enabled = update.get("enabled").and_then(|v| v.as_bool());
    let enforce_sso = update.get("enforce_sso").and_then(|v| v.as_bool());

    let config = sqlx::query_as::<_, SsoConfig>(
        "UPDATE sso_configs SET enabled = COALESCE($1, enabled), enforce_sso = COALESCE($2, enforce_sso), updated_at = NOW()
         WHERE org_id = $3
         RETURNING id, org_id, provider, issuer_url, client_id, metadata_url, enabled, enforce_sso, created_at, updated_at"
    )
    .bind(enabled)
    .bind(enforce_sso)
    .bind(org_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match config {
        Some(c) => Ok(Json(c)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ═══════════════════════════════════════════════════════════════
//  SCIM User Provisioning
// ═══════════════════════════════════════════════════════════════

pub async fn scim_create_user(
    State(pool): State<PgPool>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<ScimUserRequest>,
) -> Result<Json<ScimUser>, StatusCode> {
    let user = sqlx::query_as::<_, ScimUser>(
        "INSERT INTO scim_users (org_id, external_id, email, display_name, active, roles)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, org_id, external_id, email, display_name, active, roles, synced_at",
    )
    .bind(org_id)
    .bind(&req.external_id)
    .bind(&req.email)
    .bind(&req.display_name)
    .bind(req.active.unwrap_or(true))
    .bind(&req.roles.unwrap_or_default())
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create SCIM user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(user))
}

pub async fn scim_list_users(
    State(pool): State<PgPool>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<ScimUser>>, StatusCode> {
    let users = sqlx::query_as::<_, ScimUser>(
        "SELECT id, org_id, external_id, email, display_name, active, roles, synced_at FROM scim_users WHERE org_id = $1 ORDER BY display_name"
    )
    .bind(org_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

pub async fn scim_update_user(
    State(pool): State<PgPool>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
    Json(update): Json<serde_json::Value>,
) -> Result<Json<ScimUser>, StatusCode> {
    let active = update.get("active").and_then(|v| v.as_bool());
    let display_name = update.get("display_name").and_then(|v| v.as_str());

    let user = sqlx::query_as::<_, ScimUser>(
        "UPDATE scim_users SET active = COALESCE($1, active), display_name = COALESCE($2, display_name), synced_at = NOW()
         WHERE org_id = $3 AND id = $4
         RETURNING id, org_id, external_id, email, display_name, active, roles, synced_at"
    )
    .bind(active)
    .bind(display_name)
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(u) => Ok(Json(u)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn scim_delete_user(
    State(pool): State<PgPool>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM scim_users WHERE org_id = $1 AND id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Create database tables for SSO/SCIM
pub async fn create_tables(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sso_configs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES organizations(id) UNIQUE,
            provider VARCHAR(64) NOT NULL,
            issuer_url TEXT,
            client_id TEXT,
            metadata_url TEXT,
            enabled BOOLEAN NOT NULL DEFAULT true,
            enforce_sso BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scim_users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES organizations(id),
            external_id VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            display_name VARCHAR(255) NOT NULL,
            active BOOLEAN NOT NULL DEFAULT true,
            roles TEXT[] DEFAULT '{}',
            synced_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(org_id, external_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scim_groups (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES organizations(id),
            external_id VARCHAR(255) NOT NULL,
            display_name VARCHAR(255) NOT NULL,
            members TEXT[] DEFAULT '{}',
            synced_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(org_id, external_id)
        )",
    )
    .execute(pool)
    .await?;

    tracing::info!("SSO/SCIM tables ready");
    Ok(())
}
