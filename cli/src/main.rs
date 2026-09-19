use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod auth;
mod config;
mod mcp;
mod skill;
mod plugin;
mod client;
mod status;

#[derive(Parser)]
#[command(name = "hitechcloud", version, about = "HiTechCloud AI OS CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login with API key
    Login {
        /// API key
        #[arg(long)]
        key: String,
        /// Gateway URL (default: http://localhost:8080)
        #[arg(long, default_value = "http://localhost:8080")]
        url: String,
    },
    /// Show current configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// MCP server management
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
    /// Skill management
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Plugin management
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Export client configuration
    Client {
        #[command(subcommand)]
        action: ClientAction,
    },
    /// Check service status
    Status,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current config
    Show,
    /// Set a config value
    Set { key: String, value: String },
}

#[derive(Subcommand)]
enum McpAction {
    /// Start stdio MCP proxy
    Serve {
        /// MCP server name to proxy
        #[arg(long)]
        server: String,
    },
    /// Publish an MCP server
    Publish {
        /// Path to mcp.json manifest
        #[arg(long)]
        manifest: PathBuf,
    },
    /// List MCP servers
    List,
}

#[derive(Subcommand)]
enum SkillAction {
    /// Publish a skill
    Publish {
        /// Path to SKILL.md file
        #[arg(long)]
        file: PathBuf,
    },
    /// Search skills
    Search {
        /// Search query
        query: String,
    },
    /// List skills
    List,
}

#[derive(Subcommand)]
enum PluginAction {
    /// Publish a plugin
    Publish {
        /// Path to plugin manifest
        #[arg(long)]
        manifest: PathBuf,
    },
    /// List plugins
    List,
}

#[derive(Subcommand)]
enum ClientAction {
    /// Export config for AI clients
    Export {
        /// Target client: claude-code, cursor, vscode, windsurf, cline, roo, aider, codex, gemini, amazon-q, openai-codex
        #[arg(long)]
        target: String,
        /// Output file path
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CliConfig {
    pub api_key: Option<String>,
    pub gateway_url: Option<String>,
}

impl CliConfig {
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".hitechcloud")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn api_key(&self) -> Result<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Not logged in. Run: hitechcloud login --key <API_KEY>"))
    }

    pub fn gateway_url(&self) -> &str {
        self.gateway_url.as_deref().unwrap_or("http://localhost:8080")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login { key, url } => auth::login(key, url).await,
        Commands::Config { action } => match action {
            ConfigAction::Show => config::show(),
            ConfigAction::Set { key, value } => config::set(key, value),
        },
        Commands::Mcp { action } => match action {
            McpAction::Serve { server } => mcp::serve(server).await,
            McpAction::Publish { manifest } => mcp::publish(manifest).await,
            McpAction::List => mcp::list().await,
        },
        Commands::Skill { action } => match action {
            SkillAction::Publish { file } => skill::publish(file).await,
            SkillAction::Search { query } => skill::search(query).await,
            SkillAction::List => skill::list().await,
        },
        Commands::Plugin { action } => match action {
            PluginAction::Publish { manifest } => plugin::publish(manifest).await,
            PluginAction::List => plugin::list().await,
        },
        Commands::Client { action } => match action {
            ClientAction::Export { target, output } => client::export(target, output),
        },
        Commands::Status => status::check().await,
    }
}
