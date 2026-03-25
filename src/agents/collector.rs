use super::Agent;
use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

/// CollectorAgent fetches raw content from registered sources.
pub struct CollectorAgent;

#[async_trait]
impl Agent for CollectorAgent {
    fn name(&self) -> &str {
        "CollectorAgent"
    }

    async fn run(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        info!(agent = self.name(), "Collecting from: {}", input);
        // TODO: implement content collection
        Ok(serde_json::json!({ "articles": [] }))
    }
}
