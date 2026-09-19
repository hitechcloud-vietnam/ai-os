use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct McpConfig {
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
}

impl McpConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            port: std::env::var("MCP_GATEWAY_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://hitechcloud:hitechcloud_dev_2026@127.0.0.1:5432/hitechcloud".to_string()
            }),
            database_max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        })
    }
}
