// config/model.rs - Configuration data models
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Global configuration settings
    #[serde(default)]
    pub global: GlobalConfig,
    /// Provider configurations keyed by name (e.g., "claude", "openai")
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            providers: HashMap::new(),
        }
    }
}

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GlobalConfig {
    /// Default provider to use when none specified
    pub default_provider: String,
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_provider: String::new(),
            log_level: default_log_level(),
            timeout_seconds: default_timeout(),
            debug: false,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_timeout() -> u64 {
    60
}

/// Provider configuration - can be concrete provider or mixin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConfig {
    /// Claude API provider
    Claude(ClaudeProviderConfig),
    /// OpenAI API provider
    #[serde(rename = "openai")]
    OpenAi(OpenAiProviderConfig),
    /// Mixin (composite) provider with load balancing/fallback
    Mixin(MixinProviderConfig),
}

impl ProviderConfig {
    /// Get the provider kind for this configuration
    pub fn kind(&self) -> ProviderKind {
        match self {
            ProviderConfig::Claude(_) => ProviderKind::Claude,
            ProviderConfig::OpenAi(_) => ProviderKind::OpenAi,
            ProviderConfig::Mixin(_) => ProviderKind::Mixin,
        }
    }
}

/// Provider kind enum for type checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Claude,
    OpenAi,
    Mixin,
}

/// Claude provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClaudeProviderConfig {
    /// API key for Claude (can be overridden via env var)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// API endpoint base URL
    #[serde(default = "default_claude_base_url")]
    pub base_url: String,
    /// Default model to use
    #[serde(default = "default_claude_model")]
    pub default_model: String,
    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for ClaudeProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: default_claude_base_url(),
            default_model: default_claude_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

fn default_claude_base_url() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_claude_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_max_tokens() -> u32 {
    8192
}

fn default_temperature() -> f32 {
    0.7
}

/// OpenAI provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenAiProviderConfig {
    /// API key for OpenAI (can be overridden via env var)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// API endpoint base URL
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
    /// Default model to use
    #[serde(default = "default_openai_model")]
    pub default_model: String,
    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for OpenAiProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: default_openai_base_url(),
            default_model: default_openai_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

fn default_openai_base_url() -> String {
    "https://api.openai.com".to_string()
}

fn default_openai_model() -> String {
    "gpt-4o".to_string()
}

/// Mixin (composite) provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MixinProviderConfig {
    /// Strategy for selecting providers
    #[serde(default)]
    pub strategy: MixinStrategy,
    /// List of provider names to combine
    pub providers: Vec<String>,
    /// Weights for weighted round-robin (must match providers length)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weights: Option<Vec<f32>>,
}

impl Default for MixinProviderConfig {
    fn default() -> Self {
        Self {
            strategy: MixinStrategy::default(),
            providers: Vec::new(),
            weights: None,
        }
    }
}

/// Mixin provider selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MixinStrategy {
    /// Try providers in order until one succeeds
    Fallback,
    /// Distribute requests across providers
    RoundRobin,
    /// Distribute based on weights
    WeightedRoundRobin,
}

impl Default for MixinStrategy {
    fn default() -> Self {
        MixinStrategy::Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.global.log_level, "info");
        assert_eq!(config.global.timeout_seconds, 60);
        assert!(!config.global.debug);
    }

    #[test]
    fn test_claude_config_default() {
        let config = ClaudeProviderConfig::default();
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.default_model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_openai_config_default() {
        let config = OpenAiProviderConfig::default();
        assert_eq!(config.base_url, "https://api.openai.com");
        assert_eq!(config.default_model, "gpt-4o");
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_mixin_config_default() {
        let config = MixinProviderConfig::default();
        assert_eq!(config.strategy, MixinStrategy::Fallback);
        assert!(config.providers.is_empty());
        assert!(config.weights.is_none());
    }

    #[test]
    fn test_mixin_strategy_default() {
        let strategy: MixinStrategy = Default::default();
        assert_eq!(strategy, MixinStrategy::Fallback);
    }

    #[test]
    fn test_provider_config_kind() {
        let claude = ProviderConfig::Claude(ClaudeProviderConfig::default());
        let openai = ProviderConfig::OpenAi(OpenAiProviderConfig::default());
        let mixin = ProviderConfig::Mixin(MixinProviderConfig::default());

        assert_eq!(claude.kind(), ProviderKind::Claude);
        assert_eq!(openai.kind(), ProviderKind::OpenAi);
        assert_eq!(mixin.kind(), ProviderKind::Mixin);
    }

    #[test]
    fn test_claude_toml_parsing() {
        let toml_str = r#"
            api_key = "sk-test123"
            base_url = "https://custom.anthropic.com"
            default_model = "claude-opus-4"
            max_tokens = 4096
            temperature = 0.5
        "#;

        let config: ClaudeProviderConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.api_key, Some("sk-test123".to_string()));
        assert_eq!(config.base_url, "https://custom.anthropic.com");
        assert_eq!(config.default_model, "claude-opus-4");
        assert_eq!(config.max_tokens, 4096);
        assert!((config.temperature - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_openai_toml_parsing() {
        let toml_str = r#"
            api_key = "sk-openai-test"
            base_url = "https://api.openai.com/v1"
            default_model = "gpt-4-turbo"
            max_tokens = 2048
            temperature = 0.8
        "#;

        let config: OpenAiProviderConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.api_key, Some("sk-openai-test".to_string()));
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.default_model, "gpt-4-turbo");
        assert_eq!(config.max_tokens, 2048);
        assert!((config.temperature - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mixin_toml_parsing() {
        let toml_str = r#"
            type = "mixin"
            strategy = "round_robin"
            providers = ["claude", "openai"]
        "#;

        let config: ProviderConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        match config {
            ProviderConfig::Mixin(mixin) => {
                assert_eq!(mixin.strategy, MixinStrategy::RoundRobin);
                assert_eq!(mixin.providers, vec!["claude", "openai"]);
            }
            _ => panic!("Expected Mixin provider config"),
        }
    }

    #[test]
    fn test_full_config_toml_parsing() {
        let toml_str = r#"
            [global]
            default_provider = "claude"
            log_level = "debug"
            timeout_seconds = 30
            debug = true

            [providers.claude]
            type = "claude"
            api_key = "sk-test"
            default_model = "claude-sonnet-4-20250514"

            [providers.openai]
            type = "openai"
            api_key = "sk-openai"
            default_model = "gpt-4o"

            [providers.mixin]
            type = "mixin"
            strategy = "fallback"
            providers = ["claude", "openai"]
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.global.default_provider, "claude");
        assert_eq!(config.global.log_level, "debug");
        assert_eq!(config.global.timeout_seconds, 30);
        assert!(config.global.debug);
        assert_eq!(config.providers.len(), 3);

        assert!(matches!(
            config.providers.get("claude"),
            Some(ProviderConfig::Claude(_))
        ));
        assert!(matches!(
            config.providers.get("openai"),
            Some(ProviderConfig::OpenAi(_))
        ));
        assert!(matches!(
            config.providers.get("mixin"),
            Some(ProviderConfig::Mixin(_))
        ));
    }
}
