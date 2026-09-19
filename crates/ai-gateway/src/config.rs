use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    #[allow(dead_code)]
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            port: std::env::var("AI_GATEWAY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://hitechcloud:hitechcloud_dev_2026@127.0.0.1:5432/hitechcloud".to_string()
            }),
            database_max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "hitechcloud-dev-secret-2026".to_string()),
        })
    }
}
