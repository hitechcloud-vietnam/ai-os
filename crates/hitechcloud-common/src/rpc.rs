use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCapabilities {
    #[serde(default)]
    pub tools: Option<McpToolsCapability>,
    #[serde(default)]
    pub resources: Option<McpResourcesCapability>,
    #[serde(default)]
    pub prompts: Option<McpPromptsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsCapability {
    #[serde(default)]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourcesCapability {
    #[serde(default)]
    pub subscribe: Option<bool>,
    #[serde(default)]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptsCapability {
    #[serde(default)]
    pub list_changed: Option<bool>,
}

/// SSE Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}
