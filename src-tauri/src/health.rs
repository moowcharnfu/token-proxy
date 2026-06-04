use crate::log::{self, LogLevel};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TAG: &str = "health";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResult {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

pub fn is_anthropic_api(api_base: &str, custom_headers: &std::collections::HashMap<String, String>) -> bool {
    api_base.contains("anthropic") || custom_headers.contains_key("anthropic-version")
}

pub fn is_google_gemini_api(api_base: &str) -> bool {
    api_base.contains("generativelanguage.googleapis.com") || api_base.contains("googleapis.com")
}

pub fn is_ollama_api(api_base: &str) -> bool {
    api_base.contains("localhost:11434") || api_base.contains("127.0.0.1:11434")
}

fn li(msg: String) {
    log::log(TAG, LogLevel::Info, &msg);
}

fn le(msg: String) {
    log::log(TAG, LogLevel::Error, &msg);
}

pub async fn test_provider_health(
    api_base: &str,
    api_key: &str,
    model: Option<&str>,
    timeout_ms: u64,
    custom_headers: &std::collections::HashMap<String, String>,
) -> HealthResult {
    li("========== 健康检查开始 ==========".into());
    li(format!("API Base: {api_base}"));
    li(format!("Model: {model:?}"));
    li(format!("Timeout: {timeout_ms}ms"));
    li(format!("Custom Headers: {custom_headers:?}"));

    let api_type = if is_anthropic_api(api_base, custom_headers) {
        "Anthropic"
    } else if is_google_gemini_api(api_base) {
        "Gemini"
    } else if is_ollama_api(api_base) {
        "Ollama"
    } else {
        "OpenAI Compatible"
    };
    li(format!("检测到 API 类型: {api_type}"));

    let client = match Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            le(format!("客户端初始化失败: {e}"));
            return HealthResult {
                status: String::from("error"),
                latency_ms: None,
                error_message: Some(format!("客户端初始化失败: {}", e)),
            }
        }
    };

    let start = std::time::Instant::now();

    match api_type {
        "Anthropic" => test_anthropic_health(&client, api_base, api_key, model, custom_headers, start).await,
        "Gemini" => test_gemini_health(&client, api_base, api_key, model, start).await,
        "Ollama" => test_ollama_health(&client, api_base, model, start).await,
        _ => test_openai_compat_health(&client, api_base, api_key, model, custom_headers, start).await,
    }
}

async fn test_anthropic_health(
    client: &Client,
    api_base: &str,
    api_key: &str,
    model: Option<&str>,
    custom_headers: &std::collections::HashMap<String, String>,
    start: std::time::Instant,
) -> HealthResult {
    let url = format!("{}/v1/messages", api_base.trim_end_matches('/'));
    let model_name = model.unwrap_or("claude-3-5-sonnet-20241022");
    li(format!("[Anthropic] POST {url}"));

    let body = serde_json::json!({
        "model": model_name,
        "max_tokens": 1,
        "messages": [{"role": "user", "content": "hi"}]
    });

    let mut request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "max-tokens-3-5-sonnet-2024-07-15")
        .json(&body);

    for (key, value) in custom_headers {
        if key != "anthropic-version" {
            request = request.header(key, value);
        }
    }

    match request.send().await {
        Ok(response) => handle_response(response, start).await,
        Err(e) => handle_request_error(e, start),
    }
}

async fn test_gemini_health(
    client: &Client,
    api_base: &str,
    api_key: &str,
    model: Option<&str>,
    start: std::time::Instant,
) -> HealthResult {
    let model_name = model.unwrap_or("gemini-2.0-flash");
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        api_base.trim_end_matches('/'), model_name, api_key
    );
    li(format!("[Gemini] POST {} (key omitted)", api_base.trim_end_matches('/')));

    let body = serde_json::json!({
        "contents": [{ "parts": [{ "text": "hi" }] }]
    });

    match client.post(&url).json(&body).send().await {
        Ok(response) => handle_response(response, start).await,
        Err(e) => handle_request_error(e, start),
    }
}

async fn test_ollama_health(
    client: &Client,
    api_base: &str,
    model: Option<&str>,
    start: std::time::Instant,
) -> HealthResult {
    let url = format!("{}/api/generate", api_base.trim_end_matches('/'));
    let model_name = model.unwrap_or("llama3.2");
    li(format!("[Ollama] POST {url}"));

    let body = serde_json::json!({
        "model": model_name, "prompt": "hi", "stream": false, "max_tokens": 1
    });

    match client.post(&url).json(&body).send().await {
        Ok(response) => handle_response(response, start).await,
        Err(e) => handle_request_error(e, start),
    }
}

async fn test_openai_compat_health(
    client: &Client,
    api_base: &str,
    api_key: &str,
    model: Option<&str>,
    custom_headers: &std::collections::HashMap<String, String>,
    start: std::time::Instant,
) -> HealthResult {
    let url = format!("{}/chat/completions", api_base.trim_end_matches('/'));
    let model_name = model.unwrap_or("test-model");
    li(format!("[OpenAI] POST {url}"));

    let body = serde_json::json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1
    });

    let mut request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body);

    for (key, value) in custom_headers {
        request = request.header(key, value);
    }

    match request.send().await {
        Ok(response) => handle_response(response, start).await,
        Err(e) => handle_request_error(e, start),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

fn format_error(e: &reqwest::Error) -> String {
    if e.is_timeout() { String::from("连接超时") }
    else if e.is_connect() { String::from("无法连接，请检查网络或 API Base URL") }
    else if e.is_decode() { String::from("响应解析失败") }
    else { truncate(&e.to_string(), 100) }
}

/// 共享：处理 HTTP 响应（成功/失败）
async fn handle_response(
    response: reqwest::Response,
    start: std::time::Instant,
) -> HealthResult {
    let latency = start.elapsed().as_millis() as u64;
    let status = response.status();
    li(format!("响应: HTTP {status} ({latency}ms)"));

    if status.is_success() {
        HealthResult { status: String::from("ok"), latency_ms: Some(latency), error_message: None }
    } else {
        let error_body = response.text().await.unwrap_or_default();
        le(format!("错误响应体: {}", truncate(&error_body, 200)));
        HealthResult {
            status: String::from("error"),
            latency_ms: Some(latency),
            error_message: Some(format!("HTTP {} - {}", status.as_u16(), truncate(&error_body, 100))),
        }
    }
}

/// 共享：处理请求错误（超时、连接失败等）
fn handle_request_error(e: reqwest::Error, start: std::time::Instant) -> HealthResult {
    le(format!("请求失败: {e:?}"));
    HealthResult {
        status: String::from("error"),
        latency_ms: Some(start.elapsed().as_millis() as u64),
        error_message: Some(format_error(&e)),
    }
}