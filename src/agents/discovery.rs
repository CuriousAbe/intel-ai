use super::Agent;
use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

/// DiscoveryAgent automatically finds new information sources
/// and expands the monitored topic graph.
pub struct DiscoveryAgent;

#[async_trait]
impl Agent for DiscoveryAgent {
    fn name(&self) -> &str {
        "DiscoveryAgent"
    }

    async fn run(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        info!(agent = self.name(), "Discovering sources for: {}", input);
        // TODO: implement source discovery
        Ok(serde_json::json!({ "sources": [] }))
    }
}
