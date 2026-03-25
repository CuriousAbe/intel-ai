use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use std::env;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Entry point: use Google Custom Search if configured, otherwise DuckDuckGo HTML.
pub async fn search(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>> {
    let google_api_key = env::var("GOOGLE_SEARCH_API_KEY").ok();
    let google_cx = env::var("GOOGLE_SEARCH_ENGINE_ID").ok();

    if let (Some(api_key), Some(cx)) = (google_api_key, google_cx) {
        println!("   (使用 Google Custom Search API)");
        return search_google(client, query, max_results, &api_key, &cx).await;
    }

    println!("   (使用 DuckDuckGo 搜索)");
    search_duckduckgo(client, query, max_results).await
}

async fn search_google(
    client: &Client,
    query: &str,
    max_results: usize,
    api_key: &str,
    cx: &str,
) -> Result<Vec<SearchResult>> {
    let num = max_results.min(10);
    let url = format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&num={}",
        api_key,
        cx,
        urlencoding::encode(query),
        num
    );

    let response: serde_json::Value = client.get(&url).send().await?.json().await?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("Google Search API 错误: {}", error);
    }

    let items = match response["items"].as_array() {
        Some(items) => items,
        None => return Ok(vec![]),
    };

    let results = items
        .iter()
        .filter_map(|item| {
            let title = item["title"].as_str()?.to_string();
            let url = item["link"].as_str()?.to_string();
            let snippet = item["snippet"].as_str().unwrap_or("").to_string();
            Some(SearchResult { title, url, snippet })
        })
        .collect();

    Ok(results)
}

async fn search_duckduckgo(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>> {
    // Try DDG Lite first (simpler HTML)
    match search_ddg_lite(client, query, max_results).await {
        Ok(results) if !results.is_empty() => return Ok(results),
        _ => {}
    }

    // Fallback to DDG HTML
    search_ddg_html(client, query, max_results).await
}

async fn search_ddg_lite(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://lite.duckduckgo.com/lite/?q={}",
        urlencoding::encode(query)
    );

    let html = client
        .get(&url)
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.7,en;q=0.5")
        .send()
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&html);

    let link_sel = Selector::parse("a.result-link").unwrap();
    let snippet_sel = Selector::parse("td.result-snippet").unwrap();

    let links: Vec<_> = document.select(&link_sel).collect();
    let snippets: Vec<_> = document.select(&snippet_sel).collect();

    let mut results = Vec::new();

    for (link_el, snippet_el) in links.iter().zip(snippets.iter()) {
        let title = link_el.text().collect::<Vec<_>>().join("").trim().to_string();
        let href = link_el.value().attr("href").unwrap_or("").to_string();
        let snippet = snippet_el
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let actual_url = extract_ddg_url(&href);
        if actual_url.is_empty() || title.is_empty() {
            continue;
        }

        results.push(SearchResult {
            title,
            url: actual_url,
            snippet,
        });

        if results.len() >= max_results {
            break;
        }
    }

    Ok(results)
}

async fn search_ddg_html(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let html = client
        .get(&url)
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.7,en;q=0.5")
        .send()
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&html);

    let result_sel = Selector::parse(".result").unwrap();
    let title_sel = Selector::parse(".result__title a, .result__a").unwrap();
    let snippet_sel = Selector::parse(".result__snippet").unwrap();

    let mut results = Vec::new();

    for result_el in document.select(&result_sel) {
        let title_el = match result_el.select(&title_sel).next() {
            Some(el) => el,
            None => continue,
        };

        let title = title_el
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        let href = title_el.value().attr("href").unwrap_or("").to_string();
        let snippet = result_el
            .select(&snippet_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        let actual_url = extract_ddg_url(&href);
        if actual_url.is_empty() || title.is_empty() {
            continue;
        }

        results.push(SearchResult {
            title,
            url: actual_url,
            snippet,
        });

        if results.len() >= max_results {
            break;
        }
    }

    Ok(results)
}

/// Extract the actual destination URL from a DuckDuckGo redirect link.
/// DDG links look like: //duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&...
fn extract_ddg_url(href: &str) -> String {
    if let Some(pos) = href.find("uddg=") {
        let encoded = &href[pos + 5..];
        let encoded = match encoded.find('&') {
            Some(amp_pos) => &encoded[..amp_pos],
            None => encoded,
        };
        return urlencoding::decode(encoded)
            .unwrap_or_default()
            .into_owned();
    }

    // Already an absolute URL
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }

    String::new()
}
