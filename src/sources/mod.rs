//! Source discovery and management.
//!
//! Responsible for:
//! - Registering and categorising information sources
//! - Health-checking and scoring sources
//! - Auto-discovery of new relevant sources

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceKind {
    RssFeed,
    Website,
    Api,
    SocialMedia,
    Academic,
    Government,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub kind: SourceKind,
    pub credibility_score: f32,
    pub last_checked: Option<DateTime<Utc>>,
    pub active: bool,
}

impl Source {
    pub fn new(name: impl Into<String>, url: impl Into<String>, kind: SourceKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            url: url.into(),
            kind,
            credibility_score: 0.5,
            last_checked: None,
            active: true,
        }
    }
}

pub struct SourceRegistry {
    sources: Vec<Source>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    pub fn register(&mut self, source: Source) {
        self.sources.push(source);
    }

    pub fn active_sources(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter().filter(|s| s.active)
    }

    pub async fn health_check(&mut self) -> Result<()> {
        // TODO: ping each source and update credibility_score
        Ok(())
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
