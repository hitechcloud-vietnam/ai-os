use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  Developer Portal — Publisher & Review Management
// ═══════════════════════════════════════════════════════════════

/// Trust badge levels for published packages
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TrustBadge {
    /// Verified publisher with KYC
    Verified,
    /// Package has been signed with Ed25519
    Signed,
    /// Community contribution (not yet reviewed)
    Community,
}

impl TrustBadge {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustBadge::Verified => "verified",
            TrustBadge::Signed => "signed",
            TrustBadge::Community => "community",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "verified" => TrustBadge::Verified,
            "signed" => TrustBadge::Signed,
            _ => TrustBadge::Community,
        }
    }
}

/// Publisher profile
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Publisher {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub email: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub verified: bool,
    pub trust_level: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePublisherRequest {
    pub org_id: Uuid,
    pub name: String,
    pub email: String,
    pub website: Option<String>,
    pub description: Option<String>,
}

/// Package review status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ReviewStatus {
    Pending,
    InReview,
    Approved,
    Rejected,
    Published,
}

impl ReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewStatus::Pending => "pending",
            ReviewStatus::InReview => "in_review",
            ReviewStatus::Approved => "approved",
            ReviewStatus::Rejected => "rejected",
            ReviewStatus::Published => "published",
        }
    }
}

/// Package review record
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageReview {
    pub id: Uuid,
    pub package_type: String,
    pub package_id: Uuid,
    pub publisher_id: Uuid,
    pub version: String,
    pub status: String,
    pub reviewer_id: Option<Uuid>,
    pub review_notes: Option<String>,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitReviewRequest {
    pub package_type: String,
    pub package_id: Uuid,
    pub publisher_id: Uuid,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewActionRequest {
    pub reviewer_id: Uuid,
    pub action: String,
    pub notes: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
//  Publisher CRUD
// ═══════════════════════════════════════════════════════════════

pub async fn create_publisher(
    State(pool): State<PgPool>,
    Json(req): Json<CreatePublisherRequest>,
) -> Result<Json<Publisher>, StatusCode> {
    let publisher = sqlx::query_as::<_, Publisher>(
        "INSERT INTO publishers (org_id, name, email, website, description, trust_level)
         VALUES ($1, $2, $3, $4, $5, 'community')
         RETURNING id, org_id, name, email, website, description, verified, trust_level, created_at, updated_at"
    )
    .bind(req.org_id)
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.website)
    .bind(&req.description)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create publisher: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(publisher))
}

pub async fn list_publishers(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Publisher>>, StatusCode> {
    let publishers = sqlx::query_as::<_, Publisher>(
        "SELECT id, org_id, name, email, website, description, verified, trust_level, created_at, updated_at FROM publishers ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(publishers))
}

pub async fn get_publisher(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Publisher>, StatusCode> {
    let publisher = sqlx::query_as::<_, Publisher>(
        "SELECT id, org_id, name, email, website, description, verified, trust_level, created_at, updated_at FROM publishers WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match publisher {
        Some(p) => Ok(Json(p)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ═══════════════════════════════════════════════════════════════
//  Review Workflow
// ═══════════════════════════════════════════════════════════════

pub async fn submit_for_review(
    State(pool): State<PgPool>,
    Json(req): Json<SubmitReviewRequest>,
) -> Result<Json<PackageReview>, StatusCode> {
    let review = sqlx::query_as::<_, PackageReview>(
        "INSERT INTO package_reviews (package_type, package_id, publisher_id, version, status)
         VALUES ($1, $2, $3, $4, 'pending')
         RETURNING id, package_type, package_id, publisher_id, version, status, reviewer_id, review_notes, submitted_at, reviewed_at"
    )
    .bind(&req.package_type)
    .bind(req.package_id)
    .bind(req.publisher_id)
    .bind(&req.version)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to submit review: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        package_type = %req.package_type,
        package_id = %req.package_id,
        version = %req.version,
        "Package submitted for review"
    );

    Ok(Json(review))
}

pub async fn list_reviews(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<PackageReview>>, StatusCode> {
    let reviews = sqlx::query_as::<_, PackageReview>(
        "SELECT id, package_type, package_id, publisher_id, version, status, reviewer_id, review_notes, submitted_at, reviewed_at FROM package_reviews ORDER BY submitted_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(reviews))
}

pub async fn get_review(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<PackageReview>, StatusCode> {
    let review = sqlx::query_as::<_, PackageReview>(
        "SELECT id, package_type, package_id, publisher_id, version, status, reviewer_id, review_notes, submitted_at, reviewed_at FROM package_reviews WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match review {
        Some(r) => Ok(Json(r)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn review_action(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewActionRequest>,
) -> Result<Json<PackageReview>, StatusCode> {
    let status = match req.action.as_str() {
        "approve" => "approved",
        "reject" => "rejected",
        "publish" => "published",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let review = sqlx::query_as::<_, PackageReview>(
        "UPDATE package_reviews SET status = $1, reviewer_id = $2, review_notes = $3, reviewed_at = NOW()
         WHERE id = $4
         RETURNING id, package_type, package_id, publisher_id, version, status, reviewer_id, review_notes, submitted_at, reviewed_at"
    )
    .bind(status)
    .bind(req.reviewer_id)
    .bind(&req.notes)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match review {
        Some(r) => {
            tracing::info!(
                review_id = %id,
                action = %req.action,
                reviewer = %req.reviewer_id,
                "Review action completed"
            );
            Ok(Json(r))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create database tables for developer portal
pub async fn create_tables(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS publishers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES organizations(id),
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            website TEXT,
            description TEXT,
            verified BOOLEAN NOT NULL DEFAULT false,
            trust_level VARCHAR(32) NOT NULL DEFAULT 'community',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS package_reviews (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            package_type VARCHAR(64) NOT NULL,
            package_id UUID NOT NULL,
            publisher_id UUID NOT NULL REFERENCES publishers(id),
            version VARCHAR(64) NOT NULL,
            status VARCHAR(32) NOT NULL DEFAULT 'pending',
            reviewer_id UUID,
            review_notes TEXT,
            submitted_at TIMESTAMPTZ DEFAULT NOW(),
            reviewed_at TIMESTAMPTZ
        )",
    )
    .execute(pool)
    .await?;

    tracing::info!("Developer portal tables ready");
    Ok(())
}
