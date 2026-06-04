use std::collections::HashMap;

use crate::config::ProviderConfig;

/// API 格式枚举，用于 UI 展示
#[derive(Debug, Clone, PartialEq)]
pub enum ApiFormat {
    Anthropic,
    OpenAi,
    Gemini,
    Ollama,
}

impl ApiFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiFormat::Anthropic => "Anthropic",
            ApiFormat::OpenAi => "OpenAI Compatible",
            ApiFormat::Gemini => "Google Gemini",
            ApiFormat::Ollama => "Ollama (Local)",
        }
    }
}

/// 检测 provider 的 API 格式（复用 health.rs 的逻辑）
pub fn detect_api_format(provider: &ProviderConfig) -> ApiFormat {
    if crate::health::is_anthropic_api(&provider.apiBase, &provider.custom_headers) {
        ApiFormat::Anthropic
    } else if crate::health::is_google_gemini_api(&provider.apiBase) {
        ApiFormat::Gemini
    } else if crate::health::is_ollama_api(&provider.apiBase) {
        ApiFormat::Ollama
    } else {
        // 默认：OpenAI 兼容格式
        ApiFormat::OpenAi
    }
}

/// 生成 Claude Code 原生可读的标准环境变量
/// 始终输出 ANTHROPIC_API_KEY 和 ANTHROPIC_BASE_URL，
/// 因为 Claude Code 使用这两个变量名进行认证和路由。
pub fn generate_env_vars(provider: &ProviderConfig) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert(
        String::from("ANTHROPIC_API_KEY"),
        provider.apiKey.clone(),
    );
    vars.insert(
        String::from("ANTHROPIC_BASE_URL"),
        provider.apiBase.clone(),
    );
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider(api_base: &str, custom_headers: HashMap<String, String>) -> ProviderConfig {
        ProviderConfig {
            id: String::from("test"),
            name: String::from("Test"),
            apiBase: String::from(api_base),
            apiKey: String::from("sk-test123"),
            model: Some(String::from("test-model")),
            timeoutMs: 5000,
            custom_headers,
        }
    }

    #[test]
    fn test_detect_anthropic_by_url() {
        let p = make_provider("https://api.anthropic.com", HashMap::new());
        assert_eq!(detect_api_format(&p), ApiFormat::Anthropic);
    }

    #[test]
    fn test_detect_anthropic_by_header() {
        let mut h = HashMap::new();
        h.insert(String::from("anthropic-version"), String::from("2023-06-01"));
        let p = make_provider("https://api.sensenova.com", h);
        assert_eq!(detect_api_format(&p), ApiFormat::Anthropic);
    }

    #[test]
    fn test_detect_gemini() {
        let p = make_provider("https://generativelanguage.googleapis.com/v1beta", HashMap::new());
        assert_eq!(detect_api_format(&p), ApiFormat::Gemini);
    }

    #[test]
    fn test_detect_ollama() {
        let p = make_provider("http://localhost:11434", HashMap::new());
        assert_eq!(detect_api_format(&p), ApiFormat::Ollama);
    }

    #[test]
    fn test_detect_openai_default() {
        let p = make_provider("https://api.openai.com/v1", HashMap::new());
        assert_eq!(detect_api_format(&p), ApiFormat::OpenAi);
    }

    #[test]
    fn test_detect_deepseek() {
        let p = make_provider("https://api.deepseek.com", HashMap::new());
        assert_eq!(detect_api_format(&p), ApiFormat::OpenAi);
    }

    #[test]
    fn test_generate_env_vars() {
        let p = make_provider("https://api.deepseek.com", HashMap::new());
        let vars = generate_env_vars(&p);
        assert_eq!(vars.get("ANTHROPIC_API_KEY"), Some(&String::from("sk-test123")));
        assert_eq!(vars.get("ANTHROPIC_BASE_URL"), Some(&String::from("https://api.deepseek.com")));
        assert_eq!(vars.len(), 2);
    }
}