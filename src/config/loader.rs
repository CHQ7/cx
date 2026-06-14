// config/loader.rs - Configuration file loading

use crate::config::model::Config;
use std::path::Path;

/// Error type for configuration loading
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Load configuration from a TOML file
pub async fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let content = tokio::fs::read_to_string(path).await?;
    let config: Config = toml::from_str(&content)?;
    validate_config(&config)?;
    Ok(config)
}

/// Load configuration from a TOML string (useful for testing)
pub fn load_config_from_str(content: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(content)?;
    validate_config(&config)?;
    Ok(config)
}

/// Validate configuration for consistency
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Check that default_provider exists if specified
    if !config.global.default_provider.is_empty()
        && !config.providers.contains_key(&config.global.default_provider)
    {
        return Err(ConfigError::Validation(format!(
            "Default provider '{}' not found in providers list",
            config.global.default_provider
        )));
    }

    // Validate mixin providers reference valid providers
    for (name, provider) in &config.providers {
        if let crate::config::model::ProviderConfig::Mixin(mixin) = provider {
            for ref_provider in &mixin.providers {
                if ref_provider == name {
                    return Err(ConfigError::Validation(format!(
                        "Mixin provider '{}' references itself",
                        name
                    )));
                }
                if !config.providers.contains_key(ref_provider) {
                    return Err(ConfigError::Validation(format!(
                        "Mixin provider '{}' references unknown provider '{}'",
                        name, ref_provider
                    )));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load_config_file() {
        let toml_content = r#"
[global]
default_provider = "claude"

[providers.claude]
type = "claude"
api_key = "test-key"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = load_config(temp_file.path()).await.unwrap();
        assert_eq!(config.global.default_provider, "claude");
    }

    #[test]
    fn test_load_config_from_str() {
        let toml_content = r#"
[global]
default_provider = ""
"#;

        let config = load_config_from_str(toml_content).unwrap();
        assert!(config.global.default_provider.is_empty());
    }

    #[test]
    fn test_validate_missing_default_provider() {
        let toml_content = r#"
[global]
default_provider = "nonexistent"
"#;

        let result = load_config_from_str(toml_content);
        assert!(matches!(result, Err(ConfigError::Validation(_))));
    }

    #[test]
    fn test_validate_mixin_self_reference() {
        let toml_content = r#"
[global]
default_provider = ""

[providers.self_ref]
type = "mixin"
strategy = "fallback"
providers = ["self_ref"]
"#;

        let result = load_config_from_str(toml_content);
        assert!(matches!(result, Err(ConfigError::Validation(_))));
    }

    #[test]
    fn test_validate_mixin_unknown_reference() {
        let toml_content = r#"
[global]
default_provider = ""

[providers.mixin]
type = "mixin"
strategy = "fallback"
providers = ["unknown"]
"#;

        let result = load_config_from_str(toml_content);
        assert!(matches!(result, Err(ConfigError::Validation(_))));
    }
}
