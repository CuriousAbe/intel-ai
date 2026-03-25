mod agents;
mod analysis;
mod api;
mod config;
mod personalization;
mod pipeline;
mod sources;
mod storage;
mod verification;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("intel_ai=info".parse()?),
        )
        .init();

    info!("Intel-AI system starting...");

    // Load configuration
    let cfg = config::AppConfig::load()?;
    info!("Configuration loaded: env={}", cfg.env);

    // Start API server
    let router = api::build_router();
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
