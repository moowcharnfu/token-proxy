// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod log;
mod claude;
mod config;
mod env;
mod health;
mod provider;

use provider::ProviderManager;
use tauri::{Manager, State};
use std::sync::Mutex;

// State container for provider manager
struct ProviderManagerState(Mutex<ProviderManager>);

#[tauri::command]
fn list_providers(state: State<'_, ProviderManagerState>) -> Result<serde_json::Value, String> {
    let manager = state.0.lock().map_err(|e| e.to_string())?;

    let providers: Vec<serde_json::Value> = manager
        .list_providers()
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "apiBase": p.apiBase,
                "apiKey": p.apiKey,
                "model": p.model,
                "timeoutMs": p.timeoutMs,
                "customHeaders": p.custom_headers,
                "healthStatus": null
            })
        })
        .collect();

    Ok(serde_json::json!({
        "providers": providers,
        "active_provider": manager.get_active_provider_id()
    }))
}

#[tauri::command]
fn add_provider(
    config: config::ProviderConfig,
    state: State<'_, ProviderManagerState>,
) -> Result<(), String> {
    let mut manager = state.0.lock().map_err(|e| e.to_string())?;
    manager.add_provider(config)?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_provider(
    id: String,
    updates: std::collections::HashMap<String, serde_yaml::Value>,
    state: State<'_, ProviderManagerState>,
) -> Result<(), String> {
    let mut manager = state.0.lock().map_err(|e| e.to_string())?;
    manager.update_provider(&id, updates)?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_provider(
    id: String,
    state: State<'_, ProviderManagerState>,
) -> Result<(), String> {
    let mut manager = state.0.lock().map_err(|e| e.to_string())?;
    manager.delete_provider(&id)?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_active_provider(
    id: String,
    state: State<'_, ProviderManagerState>,
) -> Result<(), String> {
    let mut manager = state.0.lock().map_err(|e| e.to_string())?;
    manager.set_active_provider(&id)?;
    manager.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// 测试指定 provider 的连通性（async 命令，使用 AppHandle 避免 async-State 问题）
#[tauri::command]
async fn test_provider_health(
    id: String,
    app: tauri::AppHandle,
) -> Result<health::HealthResult, String> {
    let state = app.state::<ProviderManagerState>();
    let (api_base, api_key, model, timeout_ms, custom_headers) = {
        let manager = state.0.lock().map_err(|e| e.to_string())?;
        let provider = manager.get_provider_by_id(&id)
            .ok_or_else(|| format!("Provider '{}' not found", id))?;
        (
            provider.apiBase.clone(),
            provider.apiKey.clone(),
            provider.model.clone(),
            provider.timeoutMs,
            provider.custom_headers.clone(),
        )
    };

    Ok(health::test_provider_health(
        &api_base,
        &api_key,
        model.as_deref(),
        timeout_ms,
        &custom_headers,
    ).await)
}

/// 测试自定义连接参数（供编辑弹窗使用，不走 ProviderManager）
#[tauri::command]
async fn test_custom_provider_health(
    api_base: String,
    api_key: String,
    model: Option<String>,
    timeout_ms: u64,
    custom_headers: std::collections::HashMap<String, String>,
) -> Result<health::HealthResult, String> {
    Ok(health::test_provider_health(
        &api_base,
        &api_key,
        model.as_deref(),
        timeout_ms,
        &custom_headers,
    ).await)
}

/// 将指定 provider 应用到 Claude Code
#[tauri::command]
fn apply_provider_to_claude(
    id: String,
    state: State<'_, ProviderManagerState>,
) -> Result<(), String> {
    let provider = {
        let manager = state.0.lock().map_err(|e| e.to_string())?;
        manager.get_provider_by_id(&id)
            .ok_or_else(|| format!("Provider '{}' not found", id))?
            .clone()
    };
    claude::apply_to_claude(&provider).map_err(|e| e.to_string())
}

/// 从 Claude Code 移除 Token Proxy 的配置
#[tauri::command]
fn revert_claude_config() -> Result<(), String> {
    claude::revert_from_claude().map_err(|e| e.to_string())
}

/// 获取 Claude Code 集成状态
#[tauri::command]
fn get_claude_status() -> Result<serde_json::Value, String> {
    let configured = claude::is_claude_configured().map_err(|e| e.to_string())?;
    let env_vars = claude::get_applied_env_vars().unwrap_or_default();
    let applied_provider_id = claude::get_applied_provider_id();
    Ok(serde_json::json!({
        "configured": configured,
        "envVars": env_vars,
        "appliedProviderId": applied_provider_id,
    }))
}

/// 获取 provider 的标准环境变量（仅前端展示用）
#[tauri::command]
fn get_provider_env_vars(
    id: String,
    state: State<'_, ProviderManagerState>,
) -> Result<serde_json::Value, String> {
    let provider = {
        let manager = state.0.lock().map_err(|e| e.to_string())?;
        manager.get_provider_by_id(&id)
            .ok_or_else(|| format!("Provider '{}' not found", id))?
            .clone()
    };
    let format = env::detect_api_format(&provider);
    let vars = env::generate_env_vars(&provider);
    Ok(serde_json::json!({
        "format": format.as_str(),
        "vars": vars,
    }))
}

/// 读取最新的日志行
#[tauri::command]
fn get_logs(lines: Option<usize>) -> Vec<String> {
    log::read_logs(lines.unwrap_or(50))
}

/// 清空日志
#[tauri::command]
fn clear_logs() {
    log::clear_logs();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = ProviderManager::load().expect("Failed to load provider manager");

    tauri::Builder::default()
        .manage(ProviderManagerState(Mutex::new(manager)))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            list_providers,
            add_provider,
            update_provider,
            delete_provider,
            set_active_provider,
            test_provider_health,
            test_custom_provider_health,
            apply_provider_to_claude,
            revert_claude_config,
            get_claude_status,
            get_provider_env_vars,
            get_logs,
            clear_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
