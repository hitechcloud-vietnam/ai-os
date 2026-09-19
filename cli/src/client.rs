use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use crate::CliConfig;

pub fn export(target: String, output: Option<PathBuf>) -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let gateway_url = config.gateway_url();
    let _mcp_url = gateway_url.replace(":8080", ":8081");

    let (content, filename, description) = match target.to_lowercase().as_str() {
        "claude-code" | "claude_desktop" => {
            let json = serde_json::json!({
                "mcpServers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key,
                            "HITECHCLOUD_GATEWAY_URL": gateway_url
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "claude_desktop_config.json", "Claude Code / Desktop")
        }
        "cursor" => {
            let json = serde_json::json!({
                "mcpServers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "cursor_mcp.json", "Cursor")
        }
        "vscode" | "vs-code" => {
            let json = serde_json::json!({
                "servers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "mcp.json", "VS Code")
        }
        "windsurf" => {
            let json = serde_json::json!({
                "mcpServers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "windsurf_mcp.json", "Windsurf")
        }
        "cline" | "roo" => {
            let json = serde_json::json!({
                "mcpServers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "cline_mcp.json", "Cline/Roo")
        }
        "aider" => {
            let json = serde_json::json!({
                "model": "anthropic/claude-sonnet-4-20250514",
                "mcp_servers": {
                    "hitechcloud": {
                        "command": "hitechcloud mcp serve --server hitechcloud"
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, ".aider.conf.yml", "Aider")
        }
        "codex" | "openai-codex" => {
            let json = serde_json::json!({
                "mcp_servers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "codex_mcp.json", "OpenAI Codex")
        }
        "gemini" => {
            let json = serde_json::json!({
                "mcpServers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "gemini_mcp.json", "Gemini CLI")
        }
        "amazon-q" => {
            let json = serde_json::json!({
                "mcpServers": {
                    "hitechcloud": {
                        "command": "hitechcloud",
                        "args": ["mcp", "serve", "--server", "hitechcloud"],
                        "env": {
                            "HITECHCLOUD_API_KEY": api_key
                        }
                    }
                }
            });
            (serde_json::to_string_pretty(&json)?, "amazon_q_mcp.json", "Amazon Q")
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown target: {}. Supported: claude-code, cursor, vscode, windsurf, cline, roo, aider, codex, gemini, amazon-q",
                target
            ));
        }
    };

    if let Some(path) = output {
        std::fs::write(&path, &content)?;
        println!("{} Exported for {}", "✓".green().bold(), description.cyan());
        println!("  File: {}", path.display());
    } else {
        println!("{}", format!("Config for {}: {}", description, filename).bold());
        println!("{}", "─".repeat(60));
        println!("{}", content);
        println!("\n{}", format!("Tip: hitechcloud client export --target {} --output {}", target, filename).dimmed());
    }

    Ok(())
}
