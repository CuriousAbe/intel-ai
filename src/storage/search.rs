use anyhow::Result;

pub struct SearchIndex {
    // index: tantivy::Index,
}

impl SearchIndex {
    pub fn open(_path: &str) -> Result<Self> {
        // TODO: open or create tantivy index at path
        Ok(Self {})
    }
}
