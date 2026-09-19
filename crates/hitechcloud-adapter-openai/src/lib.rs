use hitechcloud_common::rpc::McpTool;
use serde::{Deserialize, Serialize};

/// OpenAI Function Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

/// OpenAI Tool definition for Chat Completions API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunction,
}

/// Convert an MCP tool to OpenAI function calling format
pub fn mcp_tool_to_openai(tool: &McpTool) -> OpenAITool {
    OpenAITool {
        tool_type: "function".to_string(),
        function: OpenAIFunction {
            name: tool.name.replace('.', "_"), // OpenAI doesn't allow dots
            description: tool.description.clone(),
            parameters: tool.input_schema.clone(),
        },
    }
}

/// Convert a list of MCP tools to OpenAI tools array
pub fn mcp_tools_to_openai(tools: &[McpTool]) -> Vec<OpenAITool> {
    tools.iter().map(|t| mcp_tool_to_openai(t)).collect()
}

/// OpenAI tool call response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_call_id: String,
    pub output: serde_json::Value,
}

/// Build an OpenAI-compatible tool execution response
pub fn build_tool_response(tool_call_id: &str, result: serde_json::Value) -> ToolCallResult {
    ToolCallResult {
        tool_call_id: tool_call_id.to_string(),
        output: result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_to_openai_conversion() {
        let tool = McpTool {
            name: "github.create_issue".to_string(),
            description: Some("Create a GitHub issue".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"}
                },
                "required": ["title"]
            })),
        };

        let openai_tool = mcp_tool_to_openai(&tool);
        assert_eq!(openai_tool.function.name, "github_create_issue");
        assert_eq!(openai_tool.tool_type, "function");
    }
}
