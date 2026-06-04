use crate::config::{AppConfig, ProviderConfig};
use std::collections::HashMap;

pub struct ProviderManager {
    config: AppConfig,
}

impl ProviderManager {
    pub fn load() -> Result<Self, crate::config::ConfigError> {
        let config = AppConfig::ensure_exists()?;
        Ok(Self { config })
    }

    pub fn save(&self) -> Result<(), crate::config::ConfigError> {
        self.config.save()
    }

    pub fn list_providers(&self) -> Vec<&ProviderConfig> {
        self.config.providers.iter().collect()
    }

    pub fn get_provider_by_id(&self, id: &str) -> Option<&ProviderConfig> {
        self.config.providers.iter().find(|p| p.id == id)
    }

    pub fn get_active_provider_id(&self) -> &str {
        &self.config.active_provider
    }

    pub fn add_provider(&mut self, config: ProviderConfig) -> Result<(), String> {
        if self.config.providers.iter().any(|p| p.id == config.id) {
            return Err(format!("Provider '{}' already exists", config.id));
        }

        // Auto-generate ID from name if empty
        let mut config = config;
        if config.id.is_empty() {
            config.id = config
                .name
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect();
        }

        self.config.providers.push(config);
        Ok(())
    }

    pub fn update_provider(
        &mut self,
        id: &str,
        updates: HashMap<String, serde_yaml::Value>,
    ) -> Result<(), String> {
        let provider = self
            .config
            .providers
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Provider '{}' not found", id))?;

        // Apply updates
        for (key, value) in updates {
            match key.as_str() {
                "name" => {
                    if let Some(s) = value.as_str() {
                        provider.name = s.to_string();
                    }
                }
                "apiBase" => {
                    if let Some(s) = value.as_str() {
                        provider.apiBase = s.to_string();
                    }
                }
                "apiKey" => {
                    if let Some(s) = value.as_str() {
                        provider.apiKey = s.to_string();
                    }
                }
                "model" => {
                    provider.model = value.as_str().map(String::from);
                }
                "timeoutMs" => {
                    if let Some(n) = value.as_i64() {
                        provider.timeoutMs = n as u64;
                    }
                }
                "customHeaders" => {
                    if let Some(map) = value.as_mapping() {
                        provider.custom_headers = map
                            .iter()
                            .filter_map(|(k, v)| {
                                Some((
                                    k.as_str()?.to_string(),
                                    v.as_str()?.to_string(),
                                ))
                            })
                            .collect();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn delete_provider(&mut self, id: &str) -> Result<(), String> {
        let pos = self
            .config
            .providers
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| format!("Provider '{}' not found", id))?;

        // If deleting active provider, switch to first available
        if self.config.active_provider == id {
            self.config.active_provider = self
                .config
                .providers
                .iter()
                .find(|p| p.id != id)
                .map(|p| p.id.clone())
                .unwrap_or_default();
        }

        self.config.providers.remove(pos);
        Ok(())
    }

    pub fn set_active_provider(&mut self, id: &str) -> Result<(), String> {
        if !self.config.providers.iter().any(|p| p.id == id) {
            return Err(format!("Provider '{}' not found", id));
        }
        self.config.active_provider = id.to_string();
        Ok(())
    }
}
