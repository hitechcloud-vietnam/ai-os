use crate::CliConfig;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub async fn publish(manifest: PathBuf) -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let url = config.gateway_url();

    let content = std::fs::read_to_string(&manifest)?;
    let plugin_data: serde_json::Value = serde_json::from_str(&content)?;

    println!("{}", "Publishing plugin...".cyan().bold());

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/registry/plugins", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&plugin_data)
        .send()
        .await?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await?;
        println!("{} Plugin published!", "✓".green().bold());
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
        .get(format!("{}/v1/registry/plugins", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if resp.status().is_success() {
        let plugins: Vec<serde_json::Value> = resp.json().await?;
        println!("{}", "Plugins".bold());
        println!("{}", "─".repeat(60));
        for p in &plugins {
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let version = p
                .get("current_version")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            println!("  {} v{}", name.cyan(), version.dimmed());
        }
        if plugins.is_empty() {
            println!("  {}", "No plugins found".dimmed());
        }
    } else {
        return Err(anyhow::anyhow!("Failed: HTTP {}", resp.status()));
    }

    Ok(())
}
