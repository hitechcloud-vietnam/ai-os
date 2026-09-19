use anyhow::Result;
use colored::Colorize;
use crate::CliConfig;

pub fn show() -> Result<()> {
    let config = CliConfig::load();
    let path = CliConfig::config_path();

    println!("{}", "HiTechCloud CLI Configuration".bold());
    println!("{}", "─".repeat(40));
    println!("  Config: {}", path.display().to_string().dimmed());
    println!(
        "  API Key: {}",
        config
            .api_key
            .as_deref()
            .map(|k| format!("{}...{}", &k[..8.min(k.len())], &k[k.len().saturating_sub(4)..]))
            .unwrap_or_else(|| "not set".red().to_string())
    );
    println!(
        "  Gateway: {}",
        config
            .gateway_url
            .as_deref()
            .unwrap_or("http://localhost:8080")
            .cyan()
    );
    Ok(())
}

pub fn set(key: String, value: String) -> Result<()> {
    let mut config = CliConfig::load();
    match key.as_str() {
        "gateway_url" | "url" => {
            config.gateway_url = Some(value.clone());
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown config key: {}. Use: gateway_url", key));
        }
    }
    config.save()?;
    println!("{} {} = {}", "✓".green(), key.cyan(), value);
    Ok(())
}
