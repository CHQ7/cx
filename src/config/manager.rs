// config/manager.rs - Configuration manager with hot-reload support

use crate::config::loader::{load_config, ConfigError};
use crate::config::model::{Config, ProviderConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration manager that handles loading and hot-reloading
#[derive(Debug, Clone)]
pub struct ConfigManager {
    inner: Arc<RwLock<ConfigManagerInner>>,
}

#[derive(Debug)]
struct ConfigManagerInner {
    config: Config,
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// Create a new config manager with default configuration
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConfigManagerInner {
                config: Config::default(),
                config_path: None,
            })),
        }
    }

    /// Load configuration from a file
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let config = load_config(&path).await?;
        let mut inner = self.inner.write().await;
        inner.config = config;
        inner.config_path = Some(path.as_ref().to_path_buf());
        Ok(())
    }

    /// Get a clone of the current configuration
    pub async fn get_config(&self) -> Config {
        self.inner.read().await.config.clone()
    }

    /// Get a specific provider configuration
    pub async fn get_provider(&self,
        name: &str,
    ) -> Option<ProviderConfig> {
        self.inner.read().await.config.providers.get(name).cloned()
    }

    /// Get the default provider configuration
    pub async fn get_default_provider(&self,
    ) -> Option<(String, ProviderConfig)> {
        let inner = self.inner.read().await;
        let default_name = &inner.config.global.default_provider;
        if default_name.is_empty() {
            return None;
        }
        inner
            .config
            .providers
            .get(default_name)
            .map(|p| (default_name.clone(), p.clone()))
    }

    /// Get all provider names
    pub async fn get_provider_names(&self,
    ) -> Vec<String> {
        self.inner
            .read()
            .await
            .config
            .providers
            .keys()
            .cloned()
            .collect()
    }

    /// Get providers of a specific kind
    pub async fn get_providers_by_kind(
        &self,
        kind: crate::config::model::ProviderKind,
    ) -> HashMap<String, ProviderConfig> {
        let inner = self.inner.read().await;
        inner
            .config
            .providers
            .iter()
            .filter(|(_, p)| p.kind() == kind)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Reload configuration from the original file
    pub async fn reload(&self,
    ) -> Result<(), ConfigError> {
        let path = {
            let inner = self.inner.read().await;
            inner.config_path.clone()
        };

        if let Some(path) = path {
            self.load_from_file(path).await
        } else {
            Err(ConfigError::Validation(
                "No configuration file loaded".to_string(),
            ))
        }
    }

    /// Update configuration programmatically
    pub async fn update_config(&self,
        f: impl FnOnce(&mut Config),
    ) {
        let mut inner = self.inner.write().await;
        f(&mut inner.config);
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::ClaudeProviderConfig;

    #[tokio::test]
    async fn test_config_manager_new() {
        let manager = ConfigManager::new();
        let config = manager.get_config().await;
        assert_eq!(config.global.log_level, "info");
    }

    #[tokio::test]
    async fn test_get_provider() {
        let manager = ConfigManager::new();

        // Initially no providers
        assert!(manager.get_provider("claude").await.is_none());

        // Add a provider
        manager
            .update_config(|config| {
                config.providers.insert(
                    "claude".to_string(),
                    ProviderConfig::Claude(ClaudeProviderConfig::default()),
                );
            })
            .await;

        // Now should be able to get it
        let provider = manager.get_provider("claude").await;
        assert!(provider.is_some());
    }

    #[tokio::test]
    async fn test_get_provider_names() {
        let manager = ConfigManager::new();

        manager
            .update_config(|config| {
                config.providers.insert(
                    "claude".to_string(),
                    ProviderConfig::Claude(ClaudeProviderConfig::default()),
                );
                config.providers.insert(
                    "openai".to_string(),
                    ProviderConfig::OpenAi(
                        crate::config::model::OpenAiProviderConfig::default(),
                    ),
                );
            })
            .await;

        let names = manager.get_provider_names().await;
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"claude".to_string()));
        assert!(names.contains(&"openai".to_string()));
    }
}
