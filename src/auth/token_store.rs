use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Scopes that must be present for the Responses API to work.
pub const REQUIRED_SCOPES: &[&str] = &["api.responses.write"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
    /// Space-separated scope string recorded at token-issuance time.
    /// `None` means the token was obtained before scope tracking was added
    /// and may be missing required scopes.
    #[serde(default)]
    pub scopes: Option<String>,
}

/// Returns `true` only when all `REQUIRED_SCOPES` are present.
/// A token with `scopes == None` (legacy) is treated as missing scopes.
pub fn has_required_scopes(token: &TokenData) -> bool {
    match &token.scopes {
        None => false,
        Some(s) => {
            let granted: Vec<&str> = s.split_whitespace().collect();
            REQUIRED_SCOPES.iter().all(|req| granted.contains(req))
        }
    }
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
