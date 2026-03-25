use super::Agent;
use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

/// PersonalizerAgent tailors intelligence output to the
/// user's interests, role, and preferred delivery format.
pub struct PersonalizerAgent;

#[async_trait]
impl Agent for PersonalizerAgent {
    fn name(&self) -> &str {
        "PersonalizerAgent"
    }

    async fn run(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        info!(agent = self.name(), "Personalizing for: {}", input);
        // TODO: implement personalization logic
        Ok(serde_json::json!({ "personalized": true, "content": "" }))
    }
}
