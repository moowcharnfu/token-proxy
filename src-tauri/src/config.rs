use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    ParseError(#[from] serde_yaml::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(alias = "api_base", alias = "apiBase")]
    pub apiBase: String,
    #[serde(alias = "api_key", alias = "apiKey")]
    pub apiKey: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_timeout", alias = "timeout_ms", alias = "timeoutMs")]
    pub timeoutMs: u64,
    #[serde(default, rename = "customHeaders", alias = "custom_headers")]
    pub custom_headers: HashMap<String, String>,
}

fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_active")]
    pub active_provider: String,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
}

fn default_active() -> String {
    String::from("deepseek")
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        let mut dir = dirs::home_dir().unwrap_or_default();
        dir.push(".token-proxy");
        dir
    }

    pub fn config_path() -> PathBuf {
        let mut path = Self::config_dir();
        path.push("config.yaml");
        path
    }

    pub fn ensure_exists() -> Result<Self, ConfigError> {
        let config_path = Self::config_path();
        let config_dir = Self::config_dir();

        // Create directory if needed
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Create default config if not exists
        if !config_path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        Self::load()
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        let content = fs::read_to_string(&path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();

        let content = serde_yaml::to_string(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    pub fn default() -> Self {
        Self {
            active_provider: String::from("deepseek"),
            providers: vec![
                // === 国内直连可用（优先） ===
                ProviderConfig {
                    id: String::from("deepseek"),
                    name: String::from("DeepSeek"),
                    apiBase: String::from("https://api.deepseek.com"),
                    apiKey: String::from(""),
                    model: Some(String::from("deepseek-chat")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                ProviderConfig {
                    id: String::from("aliyun"),
                    name: String::from("阿里通义 (DashScope)"),
                    apiBase: String::from("https://dashscope.aliyuncs.com/compatible-mode/v1"),
                    apiKey: String::from(""),
                    model: Some(String::from("qwen-plus")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                ProviderConfig {
                    id: String::from("moonshot"),
                    name: String::from("月之暗面 (Moonshot)"),
                    apiBase: String::from("https://api.moonshot.cn/v1"),
                    apiKey: String::from(""),
                    model: Some(String::from("moonshot-v1-32k")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                ProviderConfig {
                    id: String::from("zhipu"),
                    name: String::from("智谱AI (Zhipu)"),
                    apiBase: String::from("https://open.bigmodel.cn/api/paas/v4"),
                    apiKey: String::from(""),
                    model: Some(String::from("glm-4-flash")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                ProviderConfig {
                    id: String::from("sensenova"),
                    name: String::from("商汤日日新 (SenseNova)"),
                    apiBase: String::from("https://token.sensenova.cn"),
                    apiKey: String::from(""),
                    model: Some(String::from("senxin-1")),
                    timeoutMs: 10000,
                    custom_headers: {
                        let mut h = HashMap::new();
                        h.insert(String::from("anthropic-version"), String::from("2023-06-01"));
                        h
                    },
                },
                ProviderConfig {
                    id: String::from("baichuan"),
                    name: String::from("百川智能 (Baichuan)"),
                    apiBase: String::from("https://api.baichuan-ai.com/v1"),
                    apiKey: String::from(""),
                    model: Some(String::from("Baichuan3-Turbo")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                ProviderConfig {
                    id: String::from("minimax"),
                    name: String::from("MiniMax"),
                    apiBase: String::from("https://api.minimaxi.com/v1"),
                    apiKey: String::from(""),
                    model: Some(String::from("abab6.5")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                ProviderConfig {
                    id: String::from("ollama"),
                    name: String::from("Ollama (本地)"),
                    apiBase: String::from("http://localhost:11434"),
                    apiKey: String::from(""),
                    model: Some(String::from("llama3.2")),
                    timeoutMs: 30000,
                    custom_headers: HashMap::new(),
                },
                // === 国际提供商（需代理） ===
                ProviderConfig {
                    id: String::from("openai"),
                    name: String::from("OpenAI"),
                    apiBase: String::from("https://api.openai.com/v1"),
                    apiKey: String::from(""),
                    model: Some(String::from("gpt-4o")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
                // 用户自定义 Anthropic 实例 (1233)
                ProviderConfig {
                    id: String::from("1233"),
                    name: String::from("1233"),
                    apiBase: String::from("https://token.sensenova.cn"),
                    apiKey: String::from("sk-mpZvxtQf8VEzbzCeGhTakRLiX5fFwnBu"),
                    model: Some(String::from("deepseek-v4-flash")),
                    timeoutMs: 15000,
                    custom_headers: {
                        let mut h = HashMap::new();
                        h.insert(String::from("anthropic-version"), String::from("2023-06-01"));
                        h
                    },
                },
                ProviderConfig {
                    id: String::from("google-gemini"),
                    name: String::from("Google Gemini"),
                    apiBase: String::from("https://generativelanguage.googleapis.com/v1beta"),
                    apiKey: String::from(""),
                    model: Some(String::from("gemini-2.0-flash")),
                    timeoutMs: 10000,
                    custom_headers: HashMap::new(),
                },
            ],
        }
    }
}
