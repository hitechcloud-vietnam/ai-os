use anyhow::Result;
use colored::Colorize;
use crate::CliConfig;

pub async fn check() -> Result<()> {
    let config = CliConfig::load();
    let gateway_url = config.gateway_url();

    println!("{}", "HiTechCloud Service Status".bold());
    println!("{}", "─".repeat(50));

    let mcp_url = gateway_url.replace(":8080", ":8081");
    let services: Vec<(&str, &str)> = vec![
        ("AI Gateway", &gateway_url),
        ("MCP Gateway", &mcp_url),
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    for (name, url) in &services {
        let health_url = format!("{}/health", url);
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let version = body.get("version").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  {} {} v{}", "✓".green(), name.cyan(), version.dimmed());
            }
            Ok(resp) => {
                println!("  {} {} — HTTP {}", "✗".red(), name, resp.status());
            }
            Err(_) => {
                println!("  {} {} — {}", "✗".red(), name, "unreachable".red());
            }
        }
    }

    // Check Go services
    for port in [8082, 8083, 8085] {
        let name = match port {
            8082 => "Skills Service",
            8083 => "Plugin Service",
            8085 => "Admin Service",
            _ => unreachable!(),
        };
        let health_url = format!("http://localhost:{}/health", port);
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("  {} {} — port {}", "✓".green(), name.cyan(), port);
            }
            Ok(resp) => {
                println!("  {} {} — HTTP {}", "✗".red(), name, resp.status());
            }
            Err(_) => {
                println!("  {} {} — {}", "✗".red(), name, "unreachable".red());
            }
        }
    }

    // Check database
    println!("\n{}", "Infrastructure".bold());
    println!("{}", "─".repeat(50));
    let db_url = "http://localhost:5432";
    print!("  PostgreSQL: ");
    match std::net::TcpStream::connect("127.0.0.1:5432") {
        Ok(_) => println!("{} reachable", "✓".green()),
        Err(_) => println!("{} unreachable", "✗".red()),
    }

    print!("  Redis: ");
    match std::net::TcpStream::connect("127.0.0.1:6379") {
        Ok(_) => println!("{} reachable", "✓".green()),
        Err(_) => println!("{} unreachable", "✗".red()),
    }

    let _ = db_url; // suppress unused warning
    Ok(())
}
