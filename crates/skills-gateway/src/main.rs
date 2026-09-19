use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  Skills Gateway — Discovery, Serving, Versioning, Scoping
// ═══════════════════════════════════════════════════════════════

/// Skill scope
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum SkillScope {
    User,
    Org,
    Public,
}

impl std::fmt::Display for SkillScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillScope::User => write!(f, "user"),
            SkillScope::Org => write!(f, "org"),
            SkillScope::Public => write!(f, "public"),
        }
    }
}

/// Skill record
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SkillRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub content: Option<String>,
    pub version: Option<String>,
    pub scope: Option<String>,
    pub org_id: Option<Uuid>,
    pub triggers: Option<serde_json::Value>,
    pub requires_mcp: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// SKILL.md frontmatter parsed
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub requires_mcp: Vec<String>,
}

/// Parse SKILL.md content into frontmatter + body
fn parse_skill_md(content: &str) -> Result<(SkillFrontmatter, String), String> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Err("Missing frontmatter".to_string());
    }

    let rest = &content[3..];
    let end = rest.find("---").ok_or("Unclosed frontmatter")?;
    let yaml_str = &rest[..end];
    let body = rest[end + 3..].trim().to_string();

    let frontmatter: SkillFrontmatter =
        serde_yaml::from_str(yaml_str).map_err(|e| format!("Invalid YAML: {}", e))?;

    Ok((frontmatter, body))
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

// ═══════════════════════════════════════════════════════════════
//  Handlers — CRUD
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
#[allow(dead_code)]
struct ListParams {
    scope: Option<String>,
    org_id: Option<Uuid>,
    search: Option<String>,
}

async fn list_skills(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Json<Vec<SkillRecord>> {
    let mut query = String::from(
        "SELECT id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at FROM skills WHERE 1=1",
    );
    let mut bind_values: Vec<String> = vec![];

    if let Some(ref scope) = params.scope {
        bind_values.push(scope.clone());
        query.push_str(&format!(" AND scope = '{}'", scope));
    }
    if let Some(ref search) = params.search {
        let pattern = format!("%{}%", search);
        bind_values.push(pattern.clone());
        query.push_str(&format!(
            " AND (name ILIKE '{}' OR description ILIKE '{}')",
            pattern, pattern
        ));
    }

    query.push_str(" ORDER BY updated_at DESC NULLS LAST LIMIT 100");

    match sqlx::query_as::<_, SkillRecord>(&query)
        .fetch_all(&state.pool)
        .await
    {
        Ok(list) => Json(list),
        Err(e) => {
            tracing::error!("list_skills: {}", e);
            Json(vec![])
        }
    }
}

async fn get_skill(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SkillRecord>, StatusCode> {
    sqlx::query_as::<_, SkillRecord>(
        "SELECT id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at FROM skills WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(Json)
    .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct CreateSkill {
    name: String,
    description: Option<String>,
    language: Option<String>,
    content: Option<String>,
    version: Option<String>,
    scope: Option<String>,
    org_id: Option<Uuid>,
    triggers: Option<serde_json::Value>,
    requires_mcp: Option<serde_json::Value>,
}

async fn create_skill(
    State(state): State<AppState>,
    Json(req): Json<CreateSkill>,
) -> Result<(StatusCode, Json<SkillRecord>), StatusCode> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let scope = req.scope.unwrap_or_else(|| "org".to_string());
    let version = req.version.unwrap_or_else(|| "1.0.0".to_string());

    // If content is SKILL.md, parse frontmatter
    let (final_name, final_desc, final_triggers, final_mcp) = if let Some(ref content) = req.content
    {
        if content.trim_start().starts_with("---") {
            match parse_skill_md(content) {
                Ok((fm, _body)) => (
                    fm.name,
                    fm.description.or(req.description),
                    Some(serde_json::json!(fm.triggers)),
                    Some(serde_json::json!(fm.requires_mcp)),
                ),
                Err(_) => (
                    req.name.clone(),
                    req.description.clone(),
                    req.triggers.clone(),
                    req.requires_mcp.clone(),
                ),
            }
        } else {
            (
                req.name.clone(),
                req.description.clone(),
                req.triggers.clone(),
                req.requires_mcp.clone(),
            )
        }
    } else {
        (
            req.name.clone(),
            req.description.clone(),
            req.triggers.clone(),
            req.requires_mcp.clone(),
        )
    };

    sqlx::query(
        "INSERT INTO skills (id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
    )
    .bind(id)
    .bind(&final_name)
    .bind(&final_desc)
    .bind(&req.language)
    .bind(&req.content)
    .bind(&version)
    .bind(&scope)
    .bind(req.org_id)
    .bind(&final_triggers)
    .bind(&final_mcp)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let record = SkillRecord {
        id,
        name: final_name,
        description: final_desc,
        language: req.language,
        content: req.content,
        version: Some(version),
        scope: Some(scope),
        org_id: req.org_id,
        triggers: final_triggers,
        requires_mcp: final_mcp,
        created_at: Some(now),
        updated_at: Some(now),
    };

    Ok((StatusCode::CREATED, Json(record)))
}

async fn update_skill(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<SkillRecord>, StatusCode> {
    let now = chrono::Utc::now();

    // Build dynamic update
    let name = req.get("name").and_then(|v| v.as_str());
    let description = req.get("description").and_then(|v| v.as_str());
    let content = req.get("content").and_then(|v| v.as_str());

    sqlx::query(
        "UPDATE skills SET name = COALESCE($1, name), description = COALESCE($2, description), content = COALESCE($3, content), updated_at = $4 WHERE id = $5"
    )
    .bind(name)
    .bind(description)
    .bind(content)
    .bind(now)
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_skill(State(state), Path(id)).await
}

async fn delete_skill(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM skills WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

// ═══════════════════════════════════════════════════════════════
//  Handlers — Discovery & Search
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    scope: Option<String>,
    limit: Option<i64>,
}

async fn search_skills(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<Vec<SkillRecord>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let pattern = format!("%{}%", params.q);

    let query = if let Some(ref scope) = params.scope {
        sqlx::query_as::<_, SkillRecord>(
            "SELECT id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at FROM skills WHERE (name ILIKE $1 OR description ILIKE $1) AND scope = $2 ORDER BY updated_at DESC NULLS LAST LIMIT $3"
        )
        .bind(&pattern)
        .bind(scope)
        .bind(limit)
    } else {
        sqlx::query_as::<_, SkillRecord>(
            "SELECT id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at FROM skills WHERE name ILIKE $1 OR description ILIKE $1 ORDER BY updated_at DESC NULLS LAST LIMIT $2"
        )
        .bind(&pattern)
        .bind(limit)
    };

    match query.fetch_all(&state.pool).await {
        Ok(list) => Json(list),
        Err(e) => {
            tracing::error!("search_skills: {}", e);
            Json(vec![])
        }
    }
}

/// Suggest skills based on keywords (trigger matching)
async fn suggest_skills(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<Vec<serde_json::Value>> {
    let limit = params.limit.unwrap_or(5).min(20);
    let pattern = format!("%{}%", params.q);

    let results = sqlx::query_as::<_, SkillRecord>(
        "SELECT id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at FROM skills WHERE triggers::text ILIKE $1 OR name ILIKE $1 OR description ILIKE $1 ORDER BY updated_at DESC NULLS LAST LIMIT $2"
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let suggestions: Vec<serde_json::Value> = results
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "description": s.description,
                "version": s.version,
                "scope": s.scope,
                "triggers": s.triggers,
                "requires_mcp": s.requires_mcp,
            })
        })
        .collect();

    Json(suggestions)
}

/// Get skill content (SKILL.md body)
async fn get_skill_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<String, StatusCode> {
    let skill = sqlx::query_as::<_, SkillRecord>(
        "SELECT id, name, description, language, content, version, scope, org_id, triggers, requires_mcp, created_at, updated_at FROM skills WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(skill
        .content
        .unwrap_or_else(|| "No content available".to_string()))
}

// ═══════════════════════════════════════════════════════════════
//  Health
// ═══════════════════════════════════════════════════════════════

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "hitechcloud-skills-gateway"
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

    tracing::info!("Starting HiTechCloud Skills Gateway v0.1.0");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://hitechcloud:hitechcloud_dev_2026@localhost:5432/hitechcloud".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(health))
        // Skill CRUD
        .route("/sgw/v1/skills", get(list_skills).post(create_skill))
        .route(
            "/sgw/v1/skills/{id}",
            get(get_skill).put(update_skill).delete(delete_skill),
        )
        .route("/sgw/v1/skills/{id}/content", get(get_skill_content))
        // Discovery
        .route("/sgw/v1/search", get(search_skills))
        .route("/sgw/v1/suggest", get(suggest_skills))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("SKILLS_GATEWAY_PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()
        .unwrap_or(8088);

    let bind_addr = format!("0.0.0.0:{}", port);
    tracing::info!("Skills Gateway listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
