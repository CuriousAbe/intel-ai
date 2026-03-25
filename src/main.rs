mod agents;
mod analysis;
mod api;
mod auth;
mod config;
mod intelligence;
mod personalization;
mod pipeline;
mod sources;
mod storage;
mod verification;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();

    // `auth` subcommand: `cargo run -- auth [--status]`
    if args.len() >= 2 && args[1] == "auth" {
        return handle_auth_command(&args[2..]).await;
    }

    // CLI intelligence mode: `cargo run -- "topic"` or `cargo run -- --topic "topic"`
    if args.len() >= 2 {
        let topic = if args[1] == "--topic" {
            args.get(2..)
                .map(|s| s.join(" "))
                .unwrap_or_default()
        } else if !args[1].starts_with('-') {
            args[1..].join(" ")
        } else {
            String::new()
        };

        if !topic.is_empty() {
            let client = build_http_client()?;
            let api_key = resolve_api_key(&client).await?;
            return intelligence::run_intelligence_report_with_key(&topic, api_key).await;
        }
    }

    // Server mode
    run_server().await
}

/// Resolve the API key: env var > cached OAuth token > interactive OAuth flow.
async fn resolve_api_key(client: &reqwest::Client) -> Result<String> {
    // Priority 1: LLM_API_KEY env var
    if let Ok(key) = std::env::var("LLM_API_KEY") {
        if !key.is_empty() && key != "your_api_key_here" {
            return Ok(key);
        }
    }

    // Priority 2: cached OAuth token
    if let Ok(Some(token)) = auth::token_store::load_token() {
        // Check that the token has the required API scopes.
        // Old tokens obtained before scope tracking was added will be missing
        // api.responses.write; clear them and re-authenticate automatically.
        if !auth::token_store::has_required_scopes(&token) {
            eprintln!("⚠  已缓存的 Token 缺少必要的 scope（api.responses.write）。");
            eprintln!("   正在清除旧 Token，重新发起 OAuth 认证...\n");
            if let Ok(path) = auth::token_store::token_file_path() {
                let _ = std::fs::remove_file(&path);
            }
            // Fall through to interactive OAuth below
        } else if auth::token_store::is_token_valid(&token) {
            return Ok(token.access_token);
        } else {
            // Try to refresh if we have a refresh token
            if let Some(ref refresh_tok) = token.refresh_token {
                println!("🔄 Access Token 已过期，正在自动刷新...");
                match auth::oauth::refresh_access_token(client, refresh_tok).await {
                    Ok(new_token) => {
                        let _ = auth::token_store::save_token(&new_token);
                        println!("✅ Token 刷新成功\n");
                        return Ok(new_token.access_token);
                    }
                    Err(e) => {
                        eprintln!("⚠ Token 刷新失败: {}，将重新登录\n", e);
                    }
                }
            }
        }
    }

    // Priority 3: interactive OAuth flow
    println!("\n🔐 未检测到有效的 API Key，需要通过 ChatGPT 账号授权");
    let token = auth::oauth::run_oauth_flow(client).await?;
    auth::token_store::save_token(&token)?;
    println!("\n✅ 登录成功！Token 已保存到 ~/.intel-ai/auth.json\n");
    Ok(token.access_token)
}

/// Handle `cargo run -- auth [--status]`
async fn handle_auth_command(args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--status") {
        return show_auth_status();
    }

    let client = build_http_client()?;
    println!("\n🔐 开始 OAuth 认证流程...");
    let token = auth::oauth::run_oauth_flow(&client).await?;
    auth::token_store::save_token(&token)?;
    println!("\n✅ 认证成功！");

    let path = auth::token_store::token_file_path()?;
    println!("   Token 已保存到: {}", path.display());

    if let Some(expires_at) = token.expires_at {
        println!(
            "   Access Token 有效期至: {}",
            expires_at.format("%Y-%m-%d %H:%M UTC")
        );
    }
    if token.refresh_token.is_some() {
        println!("   包含 Refresh Token（用于自动续期）");
    }

    Ok(())
}

fn show_auth_status() -> Result<()> {
    match auth::token_store::load_token()? {
        None => {
            println!("\n❌ 未找到保存的 Token");
            println!("   运行 `cargo run -- auth` 进行登录\n");
        }
        Some(token) => {
            let valid = auth::token_store::is_token_valid(&token);
            println!("\n📋 认证状态:");
            println!("   状态: {}", if valid { "✅ 有效" } else { "❌ 已过期" });
            if let Some(expires_at) = token.expires_at {
                println!("   过期时间: {}", expires_at.format("%Y-%m-%d %H:%M UTC"));
            } else {
                println!("   过期时间: 未知");
            }
            println!(
                "   Refresh Token: {}",
                if token.refresh_token.is_some() { "存在" } else { "无" }
            );
            let path = auth::token_store::token_file_path()?;
            println!("   Token 文件: {}", path.display());
            println!();
        }
    }
    Ok(())
}

fn build_http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}

async fn run_server() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("intel_ai=info".parse()?),
        )
        .init();

    info!("Intel-AI system starting...");

    let cfg = config::AppConfig::load()?;
    info!("Configuration loaded: env={}", cfg.env);

    let router = api::build_router();
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
