//! Data processing pipeline.
//!
//! Stages:
//! 1. Ingest  — receive raw content from collectors
//! 2. Enrich  — extract entities, keywords, embeddings
//! 3. Dedupe  — remove near-duplicate articles
//! 4. Route   — send to verifier / analyst / storage

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawArticle {
    pub id: Uuid,
    pub source_id: Uuid,
    pub url: String,
    pub title: String,
    pub body: String,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedArticle {
    pub raw: RawArticle,
    pub entities: Vec<String>,
    pub keywords: Vec<String>,
    pub embedding: Vec<f32>,
    pub language: String,
}

pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    pub async fn ingest(&self, article: RawArticle) -> Result<EnrichedArticle> {
        // TODO: NLP enrichment, embedding generation
        Ok(EnrichedArticle {
            raw: article,
            entities: vec![],
            keywords: vec![],
            embedding: vec![],
            language: "en".to_string(),
        })
    }

    pub async fn dedupe(&self, articles: Vec<EnrichedArticle>) -> Result<Vec<EnrichedArticle>> {
        // TODO: vector similarity deduplication via Qdrant
        Ok(articles)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
