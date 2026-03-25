//! Storage layer.
//!
//! Three complementary backends:
//! - PostgreSQL (sqlx)  — structured metadata, user profiles, source registry
//! - Qdrant             — vector embeddings for semantic search & dedup
//! - Tantivy            — full-text search index

use anyhow::Result;

pub mod pg;
pub mod vector;
pub mod search;

pub use pg::PgStore;
pub use vector::VectorStore;
pub use search::SearchIndex;

pub struct Storage {
    pub pg: PgStore,
    pub vector: VectorStore,
    pub search: SearchIndex,
}

impl Storage {
    pub async fn connect(
        pg_url: &str,
        qdrant_url: &str,
        index_path: &str,
    ) -> Result<Self> {
        Ok(Self {
            pg: PgStore::connect(pg_url).await?,
            vector: VectorStore::connect(qdrant_url).await?,
            search: SearchIndex::open(index_path)?,
        })
    }
}
