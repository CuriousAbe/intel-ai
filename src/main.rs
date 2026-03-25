mod agents;
mod analysis;
mod api;
mod config;
mod intelligence;
mod personalization;
mod pipeline;
mod sources;
mod storage;
mod verification;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // CLI mode: cargo run -- "topic keyword"
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && !args[1].starts_with('-') {
        let topic = args[1..].join(" ");
        return intelligence::run_intelligence_report(&topic).await;
    }

    // Server mode
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("intel_ai=info".parse()?),
        )
        .init();

    info!("Intel-AI system starting...");

    let cfg = config::AppConfig::load()?;
    info!("Configuration loaded: env={}", cfg.env);

    let router = api::build_router();
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
