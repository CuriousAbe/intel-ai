use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
}

pub fn token_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;
    Ok(home.join(".intel-ai").join("auth.json"))
}

pub fn load_token() -> Result<Option<TokenData>> {
    let path = token_file_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let token: TokenData = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("解析 Token 文件失败: {}", e))?;
    Ok(Some(token))
}

pub fn save_token(token: &TokenData) -> Result<()> {
    let path = token_file_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(token)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn is_token_valid(token: &TokenData) -> bool {
    match token.expires_at {
        Some(expires_at) => {
            // Treat as expired if less than 60 seconds remain
            expires_at > Utc::now() + chrono::Duration::seconds(60)
        }
        None => true, // No expiry info, assume valid
    }
}
