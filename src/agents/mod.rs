//! Agent system — orchestrates all intelligence-gathering and analysis tasks.
//!
//! Agents:
//! - MetaAgent       — top-level orchestrator, decomposes goals into sub-tasks
//! - DiscoveryAgent  — finds new sources and topics automatically
//! - CollectorAgent  — fetches raw content from sources
//! - VerifierAgent   — cross-checks facts and assesses credibility
//! - AnalystAgent    — performs multi-depth analysis
//! - PersonalizerAgent — adapts output to user preferences

pub mod meta_agent;
pub mod discovery;
pub mod collector;
pub mod verifier;
pub mod analyst;
pub mod personalizer;

use anyhow::Result;
use async_trait::async_trait;

/// Common interface for all agents.
#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, input: serde_json::Value) -> Result<serde_json::Value>;
}
