use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use hitechcloud_common::{errors::AppError, models::*};
use sqlx::PgPool;

pub fn skills_router(_pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/", get(list_skills).post(create_skill))
        .route("/{id}", get(get_skill).put(update_skill).delete(delete_skill))
        .route("/{id}/versions", get(get_skill_versions))
}

pub async fn list_skills(
    State(pool): State<PgPool>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Skill>>, AppError> {
    let search = q.q.unwrap_or_default();
    let per_page = q.per_page.unwrap_or(20);
    let offset = (q.page.unwrap_or(1) - 1) * per_page;

    let skills = if search.is_empty() {
        sqlx::query_as::<_, Skill>(
            r#"SELECT id, scope, owner_id, name, description
               FROM skills ORDER BY name LIMIT $1 OFFSET $2"#,
        )
        .bind(per_page).bind(offset)
        .fetch_all(&pool).await?
    } else {
        sqlx::query_as::<_, Skill>(
            r#"SELECT id, scope, owner_id, name, description FROM skills
               WHERE to_tsvector('simple', name || ' ' || description) @@ plainto_tsquery('simple', $1)
               ORDER BY name LIMIT $2"#,
        )
        .bind(&search).bind(per_page)
        .fetch_all(&pool).await?
    };
    Ok(Json(skills))
}

pub async fn get_skill(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Skill>, AppError> {
    let skill = sqlx::query_as::<_, Skill>(
        r#"SELECT id, scope, owner_id, name, description FROM skills WHERE id = $1"#,
    )
    .bind(&id).fetch_optional(&pool).await?
    .ok_or_else(|| AppError::NotFound(format!("Skill '{}' not found", id)))?;
    Ok(Json(skill))
}

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
    .bind(&id).bind(&req.scope).bind(&req.name).bind(&req.description)
    .fetch_one(&pool).await?;
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
    .bind(&id).bind(&req.scope).bind(&req.name).bind(&req.description)
    .fetch_optional(&pool).await?
    .ok_or_else(|| AppError::NotFound(format!("Skill '{}' not found", id)))?;
    Ok(Json(skill))
}

pub async fn delete_skill(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM skills WHERE id = $1")
        .bind(&id).execute(&pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Skill '{}' not found", id)));
    }
    Ok(Json(serde_json::json!({"deleted": true, "id": id})))
}

pub async fn get_skill_versions(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Vec<SkillVersion>>, AppError> {
    let versions = sqlx::query_as::<_, SkillVersion>(
        r#"SELECT skill_id, version, content_ref, status
           FROM skill_versions WHERE skill_id = $1 ORDER BY version DESC"#,
    )
    .bind(&id).fetch_all(&pool).await?;
    Ok(Json(versions))
}
