use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;

/// Request metrics for Prometheus
#[derive(Clone)]
#[allow(dead_code)]
pub struct RequestMetrics {
    pub start_time: Instant,
    pub method: String,
    pub path: String,
    pub status: u16,
}

/// Middleware to record request metrics and add trace context
pub async fn observability_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Extract W3C TraceContext from headers
    let traceparent = request
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let trace_id = if traceparent.len() >= 32 {
        traceparent[3..35].to_string()
    } else {
        format!("{:032x}", rand::random::<u128>())
    };

    // Add trace context to spans
    let span = tracing::info_span!(
        "http_request",
        method = %method,
        path = %path,
        trace_id = %trace_id,
    );
    let _guard = span.enter();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    info!(
        method = %method,
        path = %path,
        status = status,
        duration_ms = duration.as_millis() as u64,
        trace_id = %trace_id,
        "Request completed"
    );

    // Record metrics (would be exported to Prometheus in production)
    record_request_metric(&method, &path, status, duration);

    response
}

fn record_request_metric(method: &str, path: &str, status: u16, duration: std::time::Duration) {
    // In production, this would use opentelemetry-prometheus exporter
    // For now, log structured metrics
    if status >= 400 || duration.as_millis() > 1000 {
        tracing::warn!(
            method = %method,
            path = %path,
            status = status,
            duration_ms = duration.as_millis() as u64,
            "Slow or failed request"
        );
    }
}

/// Health check metrics for infrastructure monitoring
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HealthMetrics {
    pub total_requests: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub active_connections: u32,
}

impl HealthMetrics {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            active_connections: 0,
        }
    }
}

/// Structured audit event for compliance
#[allow(dead_code)]
pub fn audit_event(
    action: &str,
    resource_type: &str,
    resource_id: &str,
    actor_id: &str,
    org_id: &str,
    metadata: serde_json::Value,
) {
    tracing::info!(
        action = %action,
        resource_type = %resource_type,
        resource_id = %resource_id,
        actor_id = %actor_id,
        org_id = %org_id,
        metadata = %metadata,
        "AUDIT"
    );
}
