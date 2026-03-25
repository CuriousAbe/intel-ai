use anyhow::Result;

pub struct PgStore {
    // pool: sqlx::PgPool,
}

impl PgStore {
    pub async fn connect(_url: &str) -> Result<Self> {
        // TODO: let pool = sqlx::PgPool::connect(url).await?;
        Ok(Self {})
    }
}
