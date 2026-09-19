use anyhow::Result;
use colored::Colorize;
use crate::CliConfig;

pub async fn login(key: String, url: String) -> Result<()> {
    // Validate API key by calling health endpoint with auth
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", url))
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let mut config = CliConfig::load();
            config.api_key = Some(key.clone());
            config.gateway_url = Some(url.clone());
            config.save()?;

            println!("{}", "✓ Login successful!".green().bold());
            println!("  Gateway: {}", url.cyan());
            println!("  API Key: {}...{}", &key[..8.min(key.len())], &key[key.len().saturating_sub(4)..]);
            println!("\n  Config saved to: {}", CliConfig::config_path().display().to_string().dimmed());
            Ok(())
        }
        Ok(r) => {
            Err(anyhow::anyhow!("Authentication failed: HTTP {}", r.status()))
        }
        Err(e) => {
            Err(anyhow::anyhow!("Connection failed: {}. Is the gateway running at {}?", e, url))
        }
    }
}
