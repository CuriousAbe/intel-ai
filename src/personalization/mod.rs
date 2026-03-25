//! Personalization engine.
//!
//! Builds and maintains a user interest model, then scores and
//! ranks intelligence reports according to individual preferences.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub name: String,
    pub topics: Vec<String>,
    pub regions: Vec<String>,
    pub preferred_depth: u8,
    pub preferred_format: DeliveryFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryFormat {
    Brief,
    Standard,
    Detailed,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedFeed {
    pub user_id: Uuid,
    pub items: Vec<FeedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub article_id: String,
    pub relevance_score: f32,
    pub summary: String,
}

pub struct PersonalizationEngine;

impl PersonalizationEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn build_feed(
        &self,
        profile: &UserProfile,
        candidates: Vec<String>,
    ) -> Result<PersonalizedFeed> {
        // TODO: score candidates against user profile using embeddings
        let items = candidates
            .into_iter()
            .map(|id| FeedItem {
                article_id: id,
                relevance_score: 0.0,
                summary: String::new(),
            })
            .collect();

        Ok(PersonalizedFeed { user_id: profile.id, items })
    }
}

impl Default for PersonalizationEngine {
    fn default() -> Self {
        Self::new()
    }
}
