use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub env: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub qdrant: QdrantConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    pub url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let cfg = config::Config::builder()
            .set_default("env", "development")?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080u16)?
            .set_default("database.url", "postgres://localhost/intel_ai")?
            .set_default("qdrant.url", "http://localhost:6334")?
            .add_source(config::Environment::with_prefix("INTEL_AI").separator("__"))
            .build()?;

        Ok(cfg.try_deserialize()?)
    }
}
