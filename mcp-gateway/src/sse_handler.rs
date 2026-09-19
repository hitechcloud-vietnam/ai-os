use axum::response::sse::{Event, KeepAlive};
use futures::stream::Stream;
use std::convert::Infallible;

use crate::McpState;

/// Create an SSE stream for MCP streaming protocol
pub fn create_sse_stream(
    _state: McpState,
) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        // Send initial connection event
        yield Ok(Event::default()
            .event("connected")
            .data(serde_json::json!({
                "server": "hitechcloud-mcp-gateway",
                "version": "0.1.0",
                "protocol": "mcp-sse"
            }).to_string()));

        // Keep-alive loop - in production this would forward events
        // from upstream MCP servers
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            yield Ok(Event::default()
                .event("ping")
                .data(serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()));
        }
    }
}
