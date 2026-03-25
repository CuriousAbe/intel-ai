use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
    /// Space-separated scope string recorded at token-issuance time.
    /// `None` means the token was obtained before scope tracking was added.
    #[serde(default)]
    pub scopes: Option<String>,
}

/// Returns `true` when the token was obtained with the correct scopes.
/// Tokens containing `api.responses.write` are legacy tokens from a broken
/// OAuth flow (OpenAI rejects that scope with `invalid_scope`) and must be
/// discarded so the user re-authenticates with the fixed scope set.
pub fn has_required_scopes(token: &TokenData) -> bool {
    match &token.scopes {
        None => false, // No scope info — treat as legacy
        Some(s) => {
            let granted: Vec<&str> = s.split_whitespace().collect();
            // Must have openid (basic validity) and must NOT carry the
            // legacy broken scope that triggers invalid_scope on OpenAI.
            granted.contains(&"openid") && !granted.contains(&"api.responses.write")
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

/// Parse the `chatgpt_account_id` from a JWT access token payload.
/// Does not verify the signature — only reads the claims.
pub fn parse_jwt_account_id(jwt: &str) -> Option<String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;

    // Try nested claim: https://api.openai.com/auth.chatgpt_account_id
    if let Some(id) = claims
        .get("https://api.openai.com/auth")
        .and_then(|v| v.get("chatgpt_account_id"))
        .and_then(|v| v.as_str())
    {
        return Some(id.to_string());
    }
    // Try top-level chatgpt_account_id
    if let Some(id) = claims.get("chatgpt_account_id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }
    // Try generic account_id
    claims
        .get("account_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
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
