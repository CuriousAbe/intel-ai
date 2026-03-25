use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

use super::IntelConfig;

/// Send collected content to LLM and return a structured intelligence report.
pub async fn analyze(
    config: &IntelConfig,
    client: &Client,
    topic: &str,
    content: &str,
) -> Result<String> {
    let url = format!(
        "{}/chat/completions",
        config.llm_api_base_url.trim_end_matches('/')
    );

    let prompt = build_prompt(topic, content);

    let body = json!({
        "model": config.llm_model,
        "messages": [
            {
                "role": "system",
                "content": "你是一名专业情报分析师，擅长从海量原始信息中提炼关键洞见，输出结构清晰、专业准确的情报摘要。"
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.5,
        "max_tokens": 2500
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.llm_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("LLM API 错误 {}: {}", status, error_body));
    }

    let resp_json: Value = response.json().await?;

    let text = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("LLM 响应格式异常:\n{}", resp_json))?
        .to_string();

    Ok(text)
}

fn build_prompt(topic: &str, content: &str) -> String {
    format!(
        r#"请基于以下关于「{topic}」的原始信息，生成一份结构化情报摘要。

## 原始信息来源

{content}

---

## 输出要求

请严格按以下结构输出情报摘要（中文）：

# 情报摘要：{topic}

## 核心要点
（列出 3-5 条最重要的信息，每条一行，以 - 开头，控制在 60 字以内）

## 关键趋势
（描述该领域当前主要趋势，150-250 字）

## 值得深入关注的方向
（列出 3 个具体方向，说明关注理由，每项约 50 字）

## 情报评估
- **信息完整度**：[高/中/低] — 简短说明
- **时效性**：[最新/较新/一般] — 简短说明
- **可信度**：[高/中/低] — 简短说明

## 信息来源
（从原始信息中提取来源标题和 URL，格式：- [标题](URL)）
"#,
        topic = topic,
        content = content
    )
}
