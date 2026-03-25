pub mod fetcher;
pub mod llm;
pub mod search;

use anyhow::Result;
use reqwest::Client;
use std::env;

pub struct IntelConfig {
    pub llm_api_key: String,
    pub llm_api_base_url: String,
    pub llm_model: String,
    pub max_search_results: usize,
    pub max_content_chars: usize,
}

impl IntelConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("LLM_API_KEY")
            .map_err(|_| anyhow::anyhow!("LLM_API_KEY 环境变量未设置，请检查 .env 文件"))?;
        Ok(Self::with_api_key(api_key))
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            llm_api_key: api_key,
            llm_api_base_url: env::var("LLM_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            llm_model: env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
            max_search_results: env::var("MAX_SEARCH_RESULTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8),
            max_content_chars: 50_000,
        }
    }
}

/// Run an intelligence report using the given API key (from env or OAuth).
pub async fn run_intelligence_report_with_key(topic: &str, api_key: String) -> Result<()> {
    let config = IntelConfig::with_api_key(api_key);
    run_report_with_config(topic, config).await
}

pub async fn run_intelligence_report(topic: &str) -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    let config = IntelConfig::from_env()?;
    run_report_with_config(topic, config).await
}

async fn run_report_with_config(topic: &str, config: IntelConfig) -> Result<()> {

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Step 1: Search
    println!("\n🔍 正在搜索: {}\n", topic);
    let results = search::search(&client, topic, config.max_search_results).await?;

    if results.is_empty() {
        anyhow::bail!("未找到相关搜索结果，请检查网络连接或尝试其他关键词");
    }

    println!("   找到 {} 个相关结果\n", results.len());

    // Step 2: Fetch content from each URL
    println!("📥 正在抓取页面内容...\n");
    let mut all_content = String::new();
    let mut fetched_sources: Vec<String> = Vec::new();

    for (i, result) in results.iter().enumerate() {
        print!("   [{}/{}] {} ... ", i + 1, results.len(), &result.title);

        match fetcher::fetch_text(&client, &result.url).await {
            Ok(text) if !text.is_empty() => {
                println!("✓");
                all_content.push_str(&format!(
                    "\n\n---\n来源标题: {}\nURL: {}\n\n{}\n",
                    result.title, result.url, text
                ));
                fetched_sources.push(format!("- [{}]({})", result.title, result.url));
            }
            _ => {
                // Fallback to snippet
                println!("⚠ 使用摘要");
                if !result.snippet.is_empty() {
                    all_content.push_str(&format!(
                        "\n\n---\n来源标题: {}\nURL: {}\n\n{}\n",
                        result.title, result.url, result.snippet
                    ));
                    fetched_sources.push(format!("- [{}]({})", result.title, result.url));
                }
            }
        }

        if all_content.len() > config.max_content_chars {
            all_content.truncate(config.max_content_chars);
            println!("\n   (内容已达上限，停止抓取)");
            break;
        }
    }

    if all_content.is_empty() {
        anyhow::bail!("未能获取任何页面内容，请检查网络连接");
    }

    // Step 3: LLM analysis
    println!("\n🤖 正在进行 AI 情报分析...\n");
    let report = llm::analyze(&config, &client, topic, &all_content).await?;

    // Step 4: Output
    let divider = "═".repeat(70);
    println!("\n{}", divider);
    println!("{}", report);
    println!("{}", divider);

    Ok(())
}
