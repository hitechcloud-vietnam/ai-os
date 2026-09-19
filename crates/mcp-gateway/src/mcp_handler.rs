use hitechcloud_common::rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

use crate::McpState;

/// Handle MCP JSON-RPC 2.0 requests
pub async fn handle_mcp_request(_state: McpState, request: JsonRpcRequest) -> JsonRpcResponse {
    tracing::info!(
        method = %request.method,
        id = ?request.id,
        "MCP JSON-RPC request"
    );

    match request.method.as_str() {
        "initialize" => JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "protocolVersion": "2025-03-26",
                "capabilities": {
                    "tools": {"listChanged": true},
                    "resources": {"subscribe": true, "listChanged": true},
                    "prompts": {"listChanged": true}
                },
                "serverInfo": {
                    "name": "hitechcloud-mcp-gateway",
                    "version": "0.1.0"
                }
            }),
        ),
        "tools/list" => {
            // Return aggregated tools from all registered MCP servers
            match crate::routing::list_all_tools(&_state).await {
                Ok(tools) => JsonRpcResponse::success(request.id.clone(), json!({"tools": tools})),
                Err(e) => JsonRpcResponse::error(
                    request.id.clone(),
                    -32000,
                    format!("Failed to list tools: {}", e),
                ),
            }
        }
        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match crate::routing::route_tool_call(&_state, tool_name, &arguments).await {
                Ok(result) => JsonRpcResponse::success(
                    request.id.clone(),
                    json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                        }]
                    }),
                ),
                Err(e) => JsonRpcResponse::error(
                    request.id.clone(),
                    -32000,
                    format!("Tool call failed: {}", e),
                ),
            }
        }
        "resources/list" => JsonRpcResponse::success(request.id.clone(), json!({"resources": []})),
        "prompts/list" => JsonRpcResponse::success(request.id.clone(), json!({"prompts": []})),
        "ping" => JsonRpcResponse::success(request.id.clone(), json!({})),
        _ => JsonRpcResponse::error(
            request.id.clone(),
            -32601,
            format!("Method not found: {}", request.method),
        ),
    }
}
