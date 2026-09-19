use crate::CliConfig;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub async fn serve(server: String) -> Result<()> {
    let config = CliConfig::load();
    let url = config.gateway_url();
    let api_key = config.api_key()?;

    println!(
        "{}",
        format!("Starting MCP stdio proxy for: {}", server)
            .cyan()
            .bold()
    );
    println!("  Gateway: {}", url.dimmed());

    // Read JSON-RPC requests from stdin, forward to MCP gateway
    let client = reqwest::Client::new();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    use std::io::{BufRead, Write};
    let reader = std::io::BufReader::new(stdin);
    let mut stdout = stdout;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Forward to MCP gateway
        let resp = client
            .post(format!("{}/mcp/v1", url.replace("8080", "8081")))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .body(line)
            .send()
            .await;

        match resp {
            Ok(r) => {
                let body = r.text().await.unwrap_or_default();
                writeln!(stdout, "{}", body)?;
                stdout.flush()?;
            }
            Err(e) => {
                let err = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32603, "message": format!("Proxy error: {}", e)},
                    "id": null
                });
                writeln!(stdout, "{}", serde_json::to_string(&err)?)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

pub async fn publish(manifest: PathBuf) -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let url = config.gateway_url();

    let content = std::fs::read_to_string(&manifest)?;
    let mcp_server: serde_json::Value = serde_json::from_str(&content)?;

    println!("{}", "Publishing MCP server...".cyan().bold());

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/registry/mcp-servers", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&mcp_server)
        .send()
        .await?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await?;
        println!("{} MCP server published!", "✓".green().bold());
        if let Some(id) = body.get("id") {
            println!("  ID: {}", id);
        }
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Publish failed: {} — {}", status, body));
    }

    Ok(())
}

pub async fn list() -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let url = config.gateway_url();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/registry/mcp-servers", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if resp.status().is_success() {
        let servers: Vec<serde_json::Value> = resp.json().await?;
        println!("{}", "MCP Servers".bold());
        println!("{}", "─".repeat(60));
        for s in &servers {
            let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let slug = s.get("slug").and_then(|v| v.as_str()).unwrap_or("?");
            println!("  {} — {}", name.cyan(), slug.dimmed());
        }
        if servers.is_empty() {
            println!("  {}", "No MCP servers found".dimmed());
        }
    } else {
        return Err(anyhow::anyhow!("Failed: HTTP {}", resp.status()));
    }

    Ok(())
}
