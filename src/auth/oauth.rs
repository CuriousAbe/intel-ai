use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::token_store::TokenData;

const AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const SCOPES: &str = "openid profile email offline_access";
const CALLBACK_PORT: u16 = 1455;

fn base64url_encode(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

fn generate_pkce_pair() -> (String, String) {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
    let code_verifier = base64url_encode(&random_bytes);

    let hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = base64url_encode(&hash);

    (code_verifier, code_challenge)
}

fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();
    base64url_encode(&random_bytes)
}

fn build_auth_url(code_challenge: &str, state: &str) -> String {
    let params = [
        ("response_type", "code"),
        ("client_id", CLIENT_ID),
        ("redirect_uri", REDIRECT_URI),
        ("scope", SCOPES),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
        ("audience", "https://api.openai.com/v1"),
        ("id_token_add_organizations", "true"),
        ("codex_cli_simplified_flow", "true"),
    ];

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", AUTH_URL, query)
}

/// Wait for the OAuth callback on localhost:1455.
/// Returns the `code` query parameter from the callback URL.
async fn wait_for_callback() -> Result<String> {
    let listener =
        tokio::net::TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT)).await?;

    let (mut stream, _addr) = listener.accept().await?;

    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // First line: "GET /auth/callback?code=...&state=... HTTP/1.1"
    let first_line = request.lines().next().unwrap_or("");
    let path_part = first_line.split_whitespace().nth(1).unwrap_or("");
    let query_str = path_part.split('?').nth(1).unwrap_or("");

    let params: HashMap<String, String> = query_str
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let raw_value = parts.next().unwrap_or("");
            let value = urlencoding::decode(raw_value)
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| raw_value.to_string());
            Some((key, value))
        })
        .collect();

    let (status_line, body) = if params.contains_key("code") {
        (
            "HTTP/1.1 200 OK".to_string(),
            "<!DOCTYPE html><html><head><meta charset='utf-8'></head><body>\
             <h2 style='color:green;font-family:sans-serif'>&#x2705; 登录成功！</h2>\
             <p style='font-family:sans-serif'>你可以关闭此窗口，返回终端继续操作。</p>\
             </body></html>"
                .to_string(),
        )
    } else {
        let err = params.get("error").cloned().unwrap_or_else(|| "未知错误".to_string());
        (
            "HTTP/1.1 400 Bad Request".to_string(),
            format!(
                "<!DOCTYPE html><html><body>\
                 <h2 style='color:red;font-family:sans-serif'>&#x274C; 认证失败</h2>\
                 <p style='font-family:sans-serif'>{}</p></body></html>",
                err
            ),
        )
    };

    let response = format!(
        "{}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    params
        .get("code")
        .cloned()
        .ok_or_else(|| {
            let err = params
                .get("error")
                .map(|s| s.as_str())
                .unwrap_or("未知错误");
            anyhow::anyhow!("OAuth 认证失败: {}", err)
        })
}

fn parse_token_response(json: &serde_json::Value) -> Result<TokenData> {
    let access_token = json["access_token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("响应中缺少 access_token: {}", json))?
        .to_string();

    let refresh_token = json["refresh_token"].as_str().map(String::from);

    let expires_at = json["expires_in"].as_u64().map(|secs| {
        Utc::now() + chrono::Duration::seconds(secs as i64)
    });

    let token_type = json["token_type"]
        .as_str()
        .unwrap_or("Bearer")
        .to_string();

    // Capture the granted scopes from the response; fall back to the
    // requested scopes so that freshly-minted tokens always pass the check.
    let scopes = json["scope"]
        .as_str()
        .map(String::from)
        .or_else(|| Some(SCOPES.to_string()));

    Ok(TokenData {
        access_token,
        refresh_token,
        expires_at,
        token_type,
        scopes,
    })
}

async fn exchange_code_for_token(
    client: &reqwest::Client,
    code: &str,
    code_verifier: &str,
) -> Result<TokenData> {
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("code", code),
        ("redirect_uri", REDIRECT_URI),
        ("code_verifier", code_verifier),
    ];

    let response = client.post(TOKEN_URL).form(&params).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Token 交换失败 {}: {}", status, body));
    }

    let json: serde_json::Value = response.json().await?;
    parse_token_response(&json)
}

/// Refresh an expired access token using the refresh token.
pub async fn refresh_access_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<TokenData> {
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", CLIENT_ID),
        ("refresh_token", refresh_token),
    ];

    let response = client.post(TOKEN_URL).form(&params).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Token 刷新失败 {}: {}", status, body));
    }

    let json: serde_json::Value = response.json().await?;
    parse_token_response(&json)
}

/// Run the full OAuth 2.0 + PKCE flow:
/// 1. Generate PKCE pair & state
/// 2. Open browser to authorization URL
/// 3. Wait for localhost callback
/// 4. Exchange code for tokens
pub async fn run_oauth_flow(client: &reqwest::Client) -> Result<TokenData> {
    let (code_verifier, code_challenge) = generate_pkce_pair();
    let state = generate_state();
    let auth_url = build_auth_url(&code_challenge, &state);

    println!("\n🌐 正在打开浏览器进行登录...");
    println!("   如果浏览器未自动打开，请手动访问：");
    println!("   {}\n", auth_url);

    if let Err(e) = open::that(&auth_url) {
        eprintln!("   无法自动打开浏览器: {}", e);
    }

    println!("⏳ 等待登录完成（在浏览器中完成 ChatGPT 登录）...\n");

    let code = wait_for_callback().await?;

    println!("🔄 正在获取 Token...");
    let token = exchange_code_for_token(client, &code, &code_verifier).await?;

    Ok(token)
}
