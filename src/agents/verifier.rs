use super::Agent;
use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

/// VerifierAgent cross-checks facts against multiple sources
/// and assigns a credibility score.
pub struct VerifierAgent;

#[async_trait]
impl Agent for VerifierAgent {
    fn name(&self) -> &str {
        "VerifierAgent"
    }

    async fn run(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        info!(agent = self.name(), "Verifying: {}", input);
        // TODO: implement fact verification
        Ok(serde_json::json!({ "credibility": 0.0, "verdict": "unverified" }))
    }
}
