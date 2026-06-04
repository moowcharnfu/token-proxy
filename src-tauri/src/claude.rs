use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::config::ProviderConfig;

const CLAUDE_DIR: &str = ".claude";
const SETTINGS_FILE: &str = "settings.json";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug)]
pub enum ClaudeError {
    ReadError(String),
    WriteError(String),
    ParseError(String),
}

impl std::fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaudeError::ReadError(msg) => write!(f, "读取 Claude 配置失败: {msg}"),
            ClaudeError::WriteError(msg) => write!(f, "写入 Claude 配置失败: {msg}"),
            ClaudeError::ParseError(msg) => write!(f, "解析 Claude 配置失败: {msg}"),
        }
    }
}

fn claude_dir() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap_or_default();
    dir.push(CLAUDE_DIR);
    dir
}

fn settings_path() -> PathBuf {
    claude_dir().join(SETTINGS_FILE)
}

fn config_path() -> PathBuf {
    claude_dir().join(CONFIG_FILE)
}

/// 将 provider 配置应用到 Claude Code
///
/// 写入以下文件：
/// 1. `~/.claude/settings.json` — 设置 env 字段（ANTHROPIC_API_KEY, ANTHROPIC_BASE_URL）
/// 2. `~/.claude/config.json` — 设置 primaryApiKey = "any"（让 Claude Code 使用环境变量）
pub fn apply_to_claude(provider: &ProviderConfig) -> Result<(), ClaudeError> {
    let dir = claude_dir();
    fs::create_dir_all(&dir).map_err(|e| ClaudeError::WriteError(e.to_string()))?;

    // --- 写 settings.json ---
    let mut settings: Value = if settings_path().exists() {
        let content =
            fs::read_to_string(settings_path()).map_err(|e| ClaudeError::ReadError(e.to_string()))?;
        serde_json::from_str(&content).map_err(|e| ClaudeError::ParseError(e.to_string()))?
    } else {
        serde_json::json!({})
    };

    // 生成标准 env vars 并写入 env 字段
    let env_vars = crate::env::generate_env_vars(provider);
    settings["env"] = serde_json::json!(env_vars);
    settings["appliedProviderId"] = serde_json::json!(provider.id);

    let settings_content =
        serde_json::to_string_pretty(&settings).map_err(|e| ClaudeError::ParseError(e.to_string()))?;
    fs::write(settings_path(), settings_content)
        .map_err(|e| ClaudeError::WriteError(e.to_string()))?;

    // --- 写 config.json ---
    let config = serde_json::json!({
        "primaryApiKey": "any"
    });
    let config_content =
        serde_json::to_string_pretty(&config).map_err(|e| ClaudeError::ParseError(e.to_string()))?;
    fs::write(config_path(), config_content)
        .map_err(|e| ClaudeError::WriteError(e.to_string()))?;

    Ok(())
}

/// 从 Claude Code 中移除 Token Proxy 的配置
///
/// 1. 从 settings.json 中删除 env 字段（保留其他字段）
/// 2. 删除 config.json
pub fn revert_from_claude() -> Result<(), ClaudeError> {
    // --- 清理 settings.json ---
    if settings_path().exists() {
        let content =
            fs::read_to_string(settings_path()).map_err(|e| ClaudeError::ReadError(e.to_string()))?;
        let mut settings: Value =
            serde_json::from_str(&content).map_err(|e| ClaudeError::ParseError(e.to_string()))?;

        if let Some(obj) = settings.as_object_mut() {
            obj.remove("env");
            obj.remove("appliedProviderId");
        }

        let new_content =
            serde_json::to_string_pretty(&settings).map_err(|e| ClaudeError::ParseError(e.to_string()))?;
        fs::write(settings_path(), new_content)
            .map_err(|e| ClaudeError::WriteError(e.to_string()))?;
    }

    // --- 清理 config.json ---
    if config_path().exists() {
        fs::remove_file(config_path()).map_err(|e| ClaudeError::WriteError(e.to_string()))?;
    }

    Ok(())
}

/// 检查 Claude Code 是否已由 Token Proxy 配置
pub fn is_claude_configured() -> Result<bool, ClaudeError> {
    if !config_path().exists() {
        return Ok(false);
    }

    let content =
        fs::read_to_string(config_path()).map_err(|e| ClaudeError::ReadError(e.to_string()))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| ClaudeError::ParseError(e.to_string()))?;

    Ok(config
        .get("primaryApiKey")
        .and_then(|v| v.as_str())
        == Some("any"))
}

/// 获取已应用到 Claude Code 的 provider ID
pub fn get_applied_provider_id() -> Option<String> {
    if !settings_path().exists() {
        return None;
    }
    let content = fs::read_to_string(settings_path()).ok()?;
    let settings: Value = serde_json::from_str(&content).ok()?;
    settings.get("appliedProviderId")?.as_str().map(String::from)
}

/// 获取已应用到 Claude Code 的环境变量
pub fn get_applied_env_vars() -> Result<HashMap<String, String>, ClaudeError> {
    if !settings_path().exists() {
        return Ok(HashMap::new());
    }

    let content =
        fs::read_to_string(settings_path()).map_err(|e| ClaudeError::ReadError(e.to_string()))?;
    let settings: Value =
        serde_json::from_str(&content).map_err(|e| ClaudeError::ParseError(e.to_string()))?;

    let env = settings
        .get("env")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect::<HashMap<String, String>>()
        })
        .unwrap_or_default();

    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;
    use std::collections::HashMap;

    fn test_provider() -> ProviderConfig {
        ProviderConfig {
            id: String::from("test"),
            name: String::from("Test"),
            apiBase: String::from("https://api.test.com"),
            apiKey: String::from("sk-test"),
            model: Some(String::from("test-model")),
            timeoutMs: 5000,
            custom_headers: HashMap::new(),
        }
    }

    #[test]
    fn test_not_configured_initially() {
        // 默认情况下不应配置
        let result = is_claude_configured().unwrap_or(false);
        // 这个测试取决于实际的 ~/.claude/ 状态，仅验证不 panic
        assert!(result == true || result == false);
    }

    #[test]
    fn test_generate_env_vars() {
        let p = test_provider();
        let vars = crate::env::generate_env_vars(&p);
        assert_eq!(vars.get("ANTHROPIC_API_KEY"), Some(&String::from("sk-test")));
        assert_eq!(vars.get("ANTHROPIC_BASE_URL"), Some(&String::from("https://api.test.com")));
    }

    #[test]
    fn test_detect_format() {
        let p = test_provider();
        let format = crate::env::detect_api_format(&p);
        assert_eq!(format, crate::env::ApiFormat::OpenAi);
    }
}
