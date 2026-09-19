use anyhow::Result;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::McpState;

#[derive(Deserialize, sqlx::FromRow)]
struct McpServerRow {
    id: String,
    #[allow(dead_code)]
    transport: String,
    namespace: String,
    name: String,
}

/// Route a tool call to the appropriate MCP server
pub async fn route_tool_call(
    state: &McpState,
    tool_name: &str,
    _arguments: &Value,
) -> Result<Value> {
    let parts: Vec<&str> = tool_name.splitn(2, '.').collect();
    let namespace = if parts.len() > 1 { parts[0] } else { "default" };
    let method = if parts.len() > 1 { parts[1] } else { tool_name };

    tracing::info!(namespace = namespace, method = method, "Routing tool call");

    let server = sqlx::query_as::<_, McpServerRow>(
        "SELECT id, transport, namespace, name FROM mcp_servers WHERE namespace = $1 AND visibility = 'public'",
    )
    .bind(namespace)
    .fetch_optional(&state.pool)
    .await?;

    match server {
        Some(srv) => {
            tracing::info!(server_id = %srv.id, "Found MCP server for namespace");
            Ok(json!({
                "server": srv.id,
                "tool": method,
                "status": "proxied",
                "message": format!("Tool '{}' routed to '{}'. Full proxy in Phase 2.", method, srv.id)
            }))
        }
        None => Err(anyhow::anyhow!(
            "No MCP server found for namespace '{}'",
            namespace
        )),
    }
}

/// List all available tools from registered MCP servers
pub async fn list_all_tools(state: &McpState) -> Result<Vec<Value>> {
    let servers = sqlx::query_as::<_, McpServerRow>(
        "SELECT id, name, namespace, transport FROM mcp_servers WHERE visibility = 'public' ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut tools = Vec::new();
    for srv in &servers {
        tools.push(json!({
            "name": format!("{}.list", srv.namespace),
            "description": format!("List tools from {} MCP server", srv.name),
            "inputSchema": {"type": "object", "properties": {}}
        }));
    }

    tools.push(json!({
        "name": "hitechcloud.registry.search",
        "description": "Search the HiTechCloud package registry",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "type": {"type": "string", "enum": ["mcp-server", "skill", "plugin"]}
            },
            "required": ["query"]
        }
    }));

    tools.push(json!({
        "name": "hitechcloud.health.check",
        "description": "Check health status of the HiTechCloud platform",
        "inputSchema": {"type": "object", "properties": {}}
    }));

    Ok(tools)
}
