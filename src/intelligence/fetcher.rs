use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

/// Fetch a URL and extract the main text content.
pub async fn fetch_text(client: &Client, url: &str) -> Result<String> {
    let response = client
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.7,en;q=0.5")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}", response.status()));
    }

    // Only process HTML content
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.contains("text/html") && !content_type.is_empty() {
        return Ok(String::new());
    }

    let html = response.text().await?;
    Ok(extract_main_text(&html))
}

/// Extract readable text from HTML, targeting main content areas.
fn extract_main_text(html: &str) -> String {
    let document = Html::parse_document(html);

    // Try content-specific selectors first (most specific → most general)
    let content_selectors = [
        "article",
        "main",
        "[role='main']",
        ".article-body",
        ".article-content",
        ".post-content",
        ".post-body",
        ".entry-content",
        ".content-body",
        ".page-content",
        "#article-content",
        "#main-content",
        "#content",
        ".content",
        "body",
    ];

    for sel_str in &content_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = document.select(&sel).next() {
                let text = collect_text(&el);
                let cleaned = clean_text(&text);
                if cleaned.len() > 150 {
                    // Limit per page to avoid bloating the LLM context
                    return if cleaned.len() > 4000 {
                        let end = cleaned
                            .char_indices()
                            .map(|(i, _)| i)
                            .take_while(|&i| i <= 4000)
                            .last()
                            .unwrap_or(0);
                        cleaned[..end].to_string()
                    } else {
                        cleaned
                    };
                }
            }
        }
    }

    String::new()
}

/// Recursively collect visible text from an element, skipping noise tags.
fn collect_text(el: &scraper::ElementRef) -> String {
    el.text().collect::<Vec<_>>().join(" ")
}

/// Remove excessive whitespace, empty lines, and very short lines.
fn clean_text(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| l.len() > 10) // skip nav items, single words, etc.
        .collect();

    // Deduplicate adjacent identical lines
    let mut deduped: Vec<&str> = Vec::new();
    for line in &lines {
        if deduped.last() != Some(line) {
            deduped.push(line);
        }
    }

    deduped.join("\n")
}
