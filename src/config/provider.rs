// config/provider.rs - Provider type definitions and traits

use crate::config::model::{
    ClaudeProviderConfig, MixinProviderConfig, MixinStrategy, OpenAiProviderConfig,
};
use serde::{Deserialize, Serialize};

/// Provider type enum for serialization/deserialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Claude,
    OpenAi,
    Mixin,
}

/// Trait for LLM providers
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Check if the provider is available (has valid credentials)
    async fn is_available(&self) -> bool;

    /// Get the default model for this provider
    fn default_model(&self) -> &str;
}

/// Concrete Claude provider implementation
#[derive(Debug, Clone)]
pub struct ClaudeProvider {
    name: String,
    config: ClaudeProviderConfig,
}

impl ClaudeProvider {
    /// Create a new Claude provider
    pub fn new(name: impl Into<String>, config: ClaudeProviderConfig) -> Self {
        Self {
            name: name.into(),
            config,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &ClaudeProviderConfig {
        &self.config
    }
}

#[async_trait::async_trait]
impl LlmProvider for ClaudeProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Claude
    }

    async fn is_available(&self) -> bool {
        // Check if API key is present (either in config or env)
        self.config.api_key.is_some()
            || std::env::var("ANTHROPIC_API_KEY").is_ok()
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }
}

/// Concrete OpenAI provider implementation
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    name: String,
    config: OpenAiProviderConfig,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider
    pub fn new(name: impl Into<String>, config: OpenAiProviderConfig) -> Self {
        Self {
            name: name.into(),
            config,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &OpenAiProviderConfig {
        &self.config
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAi
    }

    async fn is_available(&self) -> bool {
        // For local models (Ollama, etc), API key is optional
        // Check if this looks like a local model by examining the base_url
        let is_local = self.is_local();

        if is_local {
            return true; // Local models don't require API key
        }

        // For cloud services, API key is required
        self.config.api_key.is_some()
            || std::env::var("OPENAI_API_KEY").is_ok()
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }
}

impl OpenAiProvider {
    /// Check if this is a local model provider
    pub fn is_local(&self) -> bool {
        let url = self.config.base_url.to_lowercase();
        url.contains("localhost")
            || url.contains("127.0.0.1")
            || url.contains(":11434")  // Ollama
            || url.contains(":1234")   // LM Studio
    }
}

/// Mixin provider that combines multiple providers
#[derive(Debug, Clone)]
pub struct MixinProvider {
    name: String,
    config: MixinProviderConfig,
    provider_refs: Vec<String>,
}

impl MixinProvider {
    /// Create a new Mixin provider
    pub fn new(
        name: impl Into<String>,
        config: MixinProviderConfig,
    ) -> Self {
        let provider_refs = config.providers.clone();
        Self {
            name: name.into(),
            config,
            provider_refs,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &MixinProviderConfig {
        &self.config
    }

    /// Get the strategy
    pub fn strategy(&self) -> MixinStrategy {
        self.config.strategy
    }

    /// Get referenced provider names
    pub fn provider_refs(&self) -> &[String] {
        &self.provider_refs
    }
}

#[async_trait::async_trait]
impl LlmProvider for MixinProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Mixin
    }

    async fn is_available(&self) -> bool {
        // Mixin is available if it has at least one referenced provider
        !self.provider_refs.is_empty()
    }

    fn default_model(&self) -> &str {
        // Mixin doesn't have a default model, it delegates to child providers
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_provider() {
        let config = ClaudeProviderConfig::default();
        let provider = ClaudeProvider::new("claude", config);

        assert_eq!(provider.name(), "claude");
        assert_eq!(provider.provider_type(), ProviderType::Claude);
        assert_eq!(provider.default_model(), "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_openai_provider() {
        let config = OpenAiProviderConfig::default();
        let provider = OpenAiProvider::new("openai", config);

        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.provider_type(), ProviderType::OpenAi);
        assert_eq!(provider.default_model(), "gpt-4o");
    }

    #[test]
    fn test_mixin_provider() {
        let config = MixinProviderConfig {
            strategy: MixinStrategy::Fallback,
            providers: vec!["claude".to_string(), "openai".to_string()],
            weights: None,
        };
        let provider = MixinProvider::new("mixin", config);

        assert_eq!(provider.name(), "mixin");
        assert_eq!(provider.provider_type(), ProviderType::Mixin);
        assert_eq!(provider.strategy(), MixinStrategy::Fallback);
        assert_eq!(provider.provider_refs(), vec!["claude", "openai"]);
    }

    #[tokio::test]
    async fn test_claude_availability_without_key() {
        // Remove env var if present
        std::env::remove_var("ANTHROPIC_API_KEY");

        let config = ClaudeProviderConfig::default();
        let provider = ClaudeProvider::new("claude", config);

        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn test_claude_availability_with_env_key() {
        // Set env var for this test
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");

        let config = ClaudeProviderConfig::default();
        let provider = ClaudeProvider::new("claude", config);

        assert!(provider.is_available().await);

        // Clean up
        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[tokio::test]
    async fn test_claude_availability_with_config_key() {
        std::env::remove_var("ANTHROPIC_API_KEY");

        let config = ClaudeProviderConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        let provider = ClaudeProvider::new("claude", config);

        assert!(provider.is_available().await);
    }

    #[tokio::test]
    async fn test_openai_availability_without_key() {
        std::env::remove_var("OPENAI_API_KEY");

        let config = OpenAiProviderConfig::default();
        let provider = OpenAiProvider::new("openai", config);

        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn test_openai_availability_with_env_key() {
        std::env::set_var("OPENAI_API_KEY", "test-key");

        let config = OpenAiProviderConfig::default();
        let provider = OpenAiProvider::new("openai", config);

        assert!(provider.is_available().await);

        std::env::remove_var("OPENAI_API_KEY");
    }

    #[tokio::test]
    async fn test_mixin_availability() {
        let config = MixinProviderConfig {
            strategy: MixinStrategy::Fallback,
            providers: vec!["claude".to_string()],
            weights: None,
        };
        let provider = MixinProvider::new("mixin", config);

        assert!(provider.is_available().await);

        let empty_config = MixinProviderConfig {
            strategy: MixinStrategy::Fallback,
            providers: vec![],
            weights: None,
        };
        let empty_provider = MixinProvider::new("empty", empty_config);

        assert!(!empty_provider.is_available().await);
    }
}
