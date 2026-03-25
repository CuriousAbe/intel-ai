use anyhow::Result;

pub struct VectorStore {
    // client: qdrant_client::Qdrant,
}

impl VectorStore {
    pub async fn connect(_url: &str) -> Result<Self> {
        // TODO: let client = qdrant_client::Qdrant::from_url(url).build()?;
        Ok(Self {})
    }
}
