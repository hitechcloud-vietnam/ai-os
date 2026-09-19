use crate::CliConfig;
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub async fn publish(file: PathBuf) -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let url = config.gateway_url();

    let content = std::fs::read_to_string(&file)?;

    // Parse SKILL.md frontmatter
    let skill_data = if content.starts_with("---") {
        // YAML frontmatter + markdown body
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 {
            let frontmatter = parts[1];
            let body = parts[2].trim();
            let mut value: serde_json::Value = serde_yaml::from_str(frontmatter)?;
            value["body"] = serde_json::Value::String(body.to_string());
            value
        } else {
            serde_json::json!({ "content": content })
        }
    } else {
        serde_json::json!({ "content": content })
    };

    println!("{}", "Publishing skill...".cyan().bold());

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/registry/skills", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&skill_data)
        .send()
        .await?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await?;
        println!("{} Skill published!", "✓".green().bold());
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

pub async fn search(query: String) -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let url = config.gateway_url();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/registry/search", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .query(&[("q", &query)])
        .send()
        .await?;

    if resp.status().is_success() {
        let results: serde_json::Value = resp.json().await?;
        println!("{}", format!("Search results for: {}", query).bold());
        println!("{}", "─".repeat(60));

        if let Some(items) = results.as_array() {
            for item in items {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let desc = item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                println!("  {} — {}", name.cyan(), desc.dimmed());
            }
            if items.is_empty() {
                println!("  {}", "No results found".dimmed());
            }
        }
    } else {
        return Err(anyhow::anyhow!("Search failed: HTTP {}", resp.status()));
    }

    Ok(())
}

pub async fn list() -> Result<()> {
    let config = CliConfig::load();
    let api_key = config.api_key()?;
    let url = config.gateway_url();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/registry/skills", url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if resp.status().is_success() {
        let skills: Vec<serde_json::Value> = resp.json().await?;
        println!("{}", "Skills".bold());
        println!("{}", "─".repeat(60));
        for s in &skills {
            let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let version = s
                .get("current_version")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            println!("  {} v{}", name.cyan(), version.dimmed());
        }
        if skills.is_empty() {
            println!("  {}", "No skills found".dimmed());
        }
    } else {
        return Err(anyhow::anyhow!("Failed: HTTP {}", resp.status()));
    }

    Ok(())
}
