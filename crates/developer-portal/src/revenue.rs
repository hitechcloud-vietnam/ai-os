use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  Revenue Share — Marketplace Payment Distribution
// ═══════════════════════════════════════════════════════════════

/// Revenue share configuration for a plugin
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RevenueShareConfig {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub publisher_id: Uuid,
    pub platform_fee_pct: f64,
    pub publisher_share_pct: f64,
    pub price_per_call_usd: f64,
    pub currency: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRevenueShareRequest {
    pub plugin_id: Uuid,
    pub publisher_id: Uuid,
    pub price_per_call_usd: f64,
    pub platform_fee_pct: Option<f64>,
}

/// Revenue transaction record
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RevenueTransaction {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub publisher_id: Uuid,
    pub org_id: Uuid,
    pub usage_record_id: Uuid,
    pub gross_amount_usd: f64,
    pub platform_fee_usd: f64,
    pub publisher_payout_usd: f64,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Publisher earnings summary
#[derive(Debug, Serialize)]
pub struct PublisherEarnings {
    pub publisher_id: Uuid,
    pub total_earnings_usd: f64,
    pub total_transactions: i64,
    pub pending_payout_usd: f64,
    pub paid_out_usd: f64,
    pub plugins: Vec<PluginEarnings>,
}

#[derive(Debug, Serialize)]
pub struct PluginEarnings {
    pub plugin_id: Uuid,
    pub plugin_name: String,
    pub total_calls: i64,
    pub total_revenue_usd: f64,
    pub publisher_earnings_usd: f64,
}

// ═══════════════════════════════════════════════════════════════
//  Revenue Share CRUD
// ═══════════════════════════════════════════════════════════════

pub async fn create_revenue_share(
    State(pool): State<PgPool>,
    Json(req): Json<CreateRevenueShareRequest>,
) -> Result<Json<RevenueShareConfig>, StatusCode> {
    let platform_fee = req.platform_fee_pct.unwrap_or(20.0);
    let publisher_share = 100.0 - platform_fee;

    let config = sqlx::query_as::<_, RevenueShareConfig>(
        "INSERT INTO revenue_share_configs (plugin_id, publisher_id, platform_fee_pct, publisher_share_pct, price_per_call_usd, currency)
         VALUES ($1, $2, $3, $4, $5, 'USD')
         RETURNING id, plugin_id, publisher_id, platform_fee_pct, publisher_share_pct, price_per_call_usd, currency, active, created_at, updated_at"
    )
    .bind(req.plugin_id)
    .bind(req.publisher_id)
    .bind(platform_fee)
    .bind(publisher_share)
    .bind(req.price_per_call_usd)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create revenue share: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(config))
}

pub async fn list_revenue_shares(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<RevenueShareConfig>>, StatusCode> {
    let configs = sqlx::query_as::<_, RevenueShareConfig>(
        "SELECT id, plugin_id, publisher_id, platform_fee_pct, publisher_share_pct, price_per_call_usd, currency, active, created_at, updated_at FROM revenue_share_configs WHERE active = true ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(configs))
}

/// Record a revenue transaction when a paid plugin is used
pub async fn record_revenue(
    State(pool): State<PgPool>,
    Json(transaction): Json<serde_json::Value>,
) -> Result<Json<RevenueTransaction>, StatusCode> {
    let plugin_id = transaction
        .get("plugin_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let publisher_id = transaction
        .get("publisher_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let org_id = transaction
        .get("org_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let usage_record_id = transaction
        .get("usage_record_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let gross_amount = transaction
        .get("gross_amount_usd")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Look up revenue share config
    let config = sqlx::query_as::<_, RevenueShareConfig>(
        "SELECT id, plugin_id, publisher_id, platform_fee_pct, publisher_share_pct, price_per_call_usd, currency, active, created_at, updated_at FROM revenue_share_configs WHERE plugin_id = $1 AND active = true"
    )
    .bind(plugin_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let (platform_fee_pct, publisher_share_pct) = if let Some(ref c) = config {
        (c.platform_fee_pct, c.publisher_share_pct)
    } else {
        (20.0, 80.0) // Default 80/20 split
    };

    let platform_fee = gross_amount * platform_fee_pct / 100.0;
    let publisher_payout = gross_amount * publisher_share_pct / 100.0;

    let record = sqlx::query_as::<_, RevenueTransaction>(
        "INSERT INTO revenue_transactions (plugin_id, publisher_id, org_id, usage_record_id, gross_amount_usd, platform_fee_usd, publisher_payout_usd, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
         RETURNING id, plugin_id, publisher_id, org_id, usage_record_id, gross_amount_usd, platform_fee_usd, publisher_payout_usd, status, created_at"
    )
    .bind(plugin_id)
    .bind(publisher_id)
    .bind(org_id)
    .bind(usage_record_id)
    .bind(gross_amount)
    .bind(platform_fee)
    .bind(publisher_payout)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to record revenue: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(record))
}

/// Get publisher earnings summary
pub async fn get_publisher_earnings(
    State(pool): State<PgPool>,
    Path(publisher_id): Path<Uuid>,
) -> Result<Json<PublisherEarnings>, StatusCode> {
    let totals = sqlx::query_as::<_, (f64, i64, f64, f64)>(
        "SELECT COALESCE(SUM(publisher_payout_usd), 0)::DOUBLE PRECISION, COUNT(*),
                COALESCE(SUM(CASE WHEN status = 'pending' THEN publisher_payout_usd ELSE 0 END), 0)::DOUBLE PRECISION,
                COALESCE(SUM(CASE WHEN status = 'paid' THEN publisher_payout_usd ELSE 0 END), 0)::DOUBLE PRECISION
         FROM revenue_transactions WHERE publisher_id = $1"
    )
    .bind(publisher_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let plugins = sqlx::query_as::<_, (Uuid, String, i64, f64, f64)>(
        "SELECT rt.plugin_id, COALESCE(p.name, 'Unknown'), COUNT(*),
                COALESCE(SUM(rt.gross_amount_usd), 0)::DOUBLE PRECISION,
                COALESCE(SUM(rt.publisher_payout_usd), 0)::DOUBLE PRECISION
         FROM revenue_transactions rt
         LEFT JOIN plugins p ON p.id = rt.plugin_id
         WHERE rt.publisher_id = $1
         GROUP BY rt.plugin_id, p.name
         ORDER BY SUM(rt.publisher_payout_usd) DESC",
    )
    .bind(publisher_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PublisherEarnings {
        publisher_id,
        total_earnings_usd: totals.0,
        total_transactions: totals.1,
        pending_payout_usd: totals.2,
        paid_out_usd: totals.3,
        plugins: plugins
            .into_iter()
            .map(|p| PluginEarnings {
                plugin_id: p.0,
                plugin_name: p.1,
                total_calls: p.2,
                total_revenue_usd: p.3,
                publisher_earnings_usd: p.4,
            })
            .collect(),
    }))
}

/// Create database tables for revenue share
pub async fn create_tables(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS revenue_share_configs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            plugin_id UUID NOT NULL REFERENCES plugins(id),
            publisher_id UUID NOT NULL,
            platform_fee_pct NUMERIC(5, 2) NOT NULL DEFAULT 20.00,
            publisher_share_pct NUMERIC(5, 2) NOT NULL DEFAULT 80.00,
            price_per_call_usd NUMERIC(12, 6) NOT NULL DEFAULT 0.0,
            currency VARCHAR(8) NOT NULL DEFAULT 'USD',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS revenue_transactions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            plugin_id UUID NOT NULL,
            publisher_id UUID NOT NULL,
            org_id UUID NOT NULL,
            usage_record_id UUID,
            gross_amount_usd NUMERIC(12, 6) NOT NULL,
            platform_fee_usd NUMERIC(12, 6) NOT NULL,
            publisher_payout_usd NUMERIC(12, 6) NOT NULL,
            status VARCHAR(32) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_revenue_publisher ON revenue_transactions(publisher_id, created_at)"
    )
    .execute(pool)
    .await?;

    tracing::info!("Revenue share tables ready");
    Ok(())
}
