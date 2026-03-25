//! Analysis engine — four-level deep analysis.
//!
//! Level 1 (L1) — Summary          : concise TL;DR
//! Level 2 (L2) — Context          : background, related events
//! Level 3 (L3) — Implications     : who is affected and how
//! Level 4 (L4) — Strategic Forecast: long-term outlook, weak signals

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisDepth {
    L1,
    L2,
    L3,
    L4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub article_id: String,
    pub depth: AnalysisDepth,
    pub summary: String,
    pub context: Option<String>,
    pub implications: Option<String>,
    pub forecast: Option<String>,
    pub confidence: f32,
}

pub struct AnalysisEngine;

impl AnalysisEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyse(&self, content: &str, depth: AnalysisDepth) -> Result<AnalysisReport> {
        // TODO: call LLM with depth-specific prompts
        Ok(AnalysisReport {
            article_id: String::new(),
            depth,
            summary: content.chars().take(200).collect(),
            context: None,
            implications: None,
            forecast: None,
            confidence: 0.0,
        })
    }
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}
