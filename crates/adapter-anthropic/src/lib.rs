use hitechcloud_common::rpc::McpTool;
use serde::{Deserialize, Serialize};

/// Anthropic SKILL.md frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub requires_mcp: Option<Vec<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Parse SKILL.md content into frontmatter + body
pub fn parse_skill_md(content: &str) -> anyhow::Result<(SkillFrontmatter, String)> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Err(anyhow::anyhow!("Missing frontmatter delimiter"));
    }

    let rest = &content[3..];
    let end = rest
        .find("---")
        .ok_or_else(|| anyhow::anyhow!("Unterminated frontmatter"))?;

    let yaml_str = &rest[..end];
    let body = rest[end + 3..].trim().to_string();

    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_str)?;
    Ok((frontmatter, body))
}

/// Convert MCP tool to Anthropic tool_use format
pub fn mcp_tool_to_anthropic(tool: &McpTool) -> serde_json::Value {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.input_schema.clone().unwrap_or(serde_json::json!({
            "type": "object",
            "properties": {}
        }))
    })
}

/// Generate .mcp.json configuration for Claude Code
pub fn generate_mcp_json(server_id: &str, transport: &str, endpoint: &str) -> serde_json::Value {
    match transport {
        "sse" => serde_json::json!({
            "mcpServers": {
                server_id: {
                    "transport": "sse",
                    "url": endpoint,
                }
            }
        }),
        "stdio" => serde_json::json!({
            "mcpServers": {
                server_id: {
                    "command": "hitechcloud",
                    "args": ["mcp", "serve", "--server", server_id],
                }
            }
        }),
        _ => serde_json::json!({
            "mcpServers": {
                server_id: {
                    "transport": "streamable-http",
                    "url": endpoint,
                }
            }
        }),
    }
}
