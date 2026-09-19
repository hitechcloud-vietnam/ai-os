use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Option<Uuid>,
    pub provider: String,
    pub model_name: Option<String>,
    pub resource_type: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cached_tokens: i32,
    pub total_cost_usd: f64,
    pub billing_period: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordUsageRequest {
    pub org_id: Uuid,
    pub user_id: Option<Uuid>,
    pub provider: String,
    pub model_name: Option<String>,
    pub resource_type: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cached_tokens: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub org_id: Uuid,
    pub billing_period: String,
    pub total_cost_usd: f64,
    pub total_calls: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub by_resource: Vec<ResourceUsage>,
}

#[derive(Debug, Serialize)]
pub struct ResourceUsage {
    pub resource_type: String,
    pub count: i64,
    pub total_cost_usd: f64,
}

/// Record a usage event
pub async fn record_usage(
    State(pool): State<PgPool>,
    Json(req): Json<RecordUsageRequest>,
) -> Result<Json<UsageRecord>, StatusCode> {
    let billing_period = chrono::Utc::now().format("%Y-%m").to_string();

    // Look up pricing if model is specified
    let unit_price: f64 = if let Some(ref model) = req.model_name {
        let result = sqlx::query_scalar(
            "SELECT COALESCE(input_price_per_million, 0)::DOUBLE PRECISION FROM model_pricing WHERE model_name = $1"
        )
        .bind(model)
        .fetch_optional(&pool)
        .await;
        match result {
            Ok(Some(price)) => price,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let input = req.input_tokens.unwrap_or(0) as f64;
    let output = req.output_tokens.unwrap_or(0) as f64;
    let total_cost = (input + output) * unit_price / 1_000_000.0;

    let record = sqlx::query_as::<_, UsageRecord>(
        "INSERT INTO usage_records (org_id, user_id, provider, model_name, resource_type, input_tokens, output_tokens, cached_tokens, effective_unit_price, total_cost_usd, metadata, billing_period)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         RETURNING id, org_id, user_id, provider, model_name, resource_type, input_tokens, output_tokens, cached_tokens, total_cost_usd::DOUBLE PRECISION, billing_period"
    )
    .bind(req.org_id)
    .bind(req.user_id)
    .bind(&req.provider)
    .bind(&req.model_name)
    .bind(&req.resource_type)
    .bind(req.input_tokens.unwrap_or(0))
    .bind(req.output_tokens.unwrap_or(0))
    .bind(req.cached_tokens.unwrap_or(0))
    .bind(unit_price)
    .bind(total_cost)
    .bind(req.metadata.unwrap_or(serde_json::json!({})))
    .bind(&billing_period)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update budget quota
    let _ = sqlx::query(
        "INSERT INTO budget_quotas (org_id, current_usage_usd, billing_period)
         VALUES ($1, $2, $3)
         ON CONFLICT (org_id) DO UPDATE SET current_usage_usd = budget_quotas.current_usage_usd + $2, updated_at = NOW()"
    )
    .bind(req.org_id)
    .bind(total_cost)
    .bind(&billing_period)
    .execute(&pool)
    .await;

    Ok(Json(record))
}

/// Get usage summary for an org
pub async fn get_usage_summary(
    State(pool): State<PgPool>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<UsageSummary>, StatusCode> {
    let billing_period = chrono::Utc::now().format("%Y-%m").to_string();

    let totals = sqlx::query_as::<_, (f64, i64, i64, i64)>(
        "SELECT COALESCE(SUM(total_cost_usd), 0)::DOUBLE PRECISION, COUNT(*), COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0)
         FROM usage_records WHERE org_id = $1 AND billing_period = $2"
    )
    .bind(org_id)
    .bind(&billing_period)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let by_resource = sqlx::query_as::<_, (String, i64, f64)>(
        "SELECT resource_type, COUNT(*), COALESCE(SUM(total_cost_usd), 0)::DOUBLE PRECISION
         FROM usage_records WHERE org_id = $1 AND billing_period = $2
         GROUP BY resource_type ORDER BY SUM(total_cost_usd) DESC"
    )
    .bind(org_id)
    .bind(&billing_period)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UsageSummary {
        org_id,
        billing_period,
        total_cost_usd: totals.0,
        total_calls: totals.1,
        total_input_tokens: totals.2,
        total_output_tokens: totals.3,
        by_resource: by_resource
            .into_iter()
            .map(|r| ResourceUsage {
                resource_type: r.0,
                count: r.1,
                total_cost_usd: r.2,
            })
            .collect(),
    }))
}

/// List model pricing
pub async fn list_pricing(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let prices = sqlx::query_as::<_, (String, String, f64, f64, f64)>(
        "SELECT provider, model_name, input_price_per_million::DOUBLE PRECISION, output_price_per_million::DOUBLE PRECISION, COALESCE(cache_read_price_per_million, 0)::DOUBLE PRECISION FROM model_pricing ORDER BY provider, model_name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        prices
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "provider": p.0,
                    "model_name": p.1,
                    "input_price_per_million": p.2,
                    "output_price_per_million": p.3,
                    "cache_read_price_per_million": p.4,
                })
            })
            .collect(),
    ))
}
