use clap::Subcommand;
use std::path::Path;
use crate::config::{ConfigManager, ProviderConfig, LlmProvider};
use crate::config::provider::{ClaudeProvider, OpenAiProvider};

/// 配置管理子命令
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// 列出所有配置的 provider
    List,

    /// 显示当前配置详情
    Show,

    /// 切换到指定的 provider
    Switch {
        /// Provider 名称
        name: String,
    },

    /// 测试指定 provider 的连通性
    Test {
        /// Provider 名称 (可选，默认使用当前 provider)
        name: Option<String>,
    },

    /// 重新加载配置文件
    Reload,
}

/// 生成示例配置文件内容
pub fn create_example_config() -> String {
    r#"# CX 配置文件示例
# 配置文件位置: ~/.cx/config.toml

[global]
# 默认使用的 provider
default_provider = "claude"
# 日志级别: trace, debug, info, warn, error
log_level = "info"
# 请求超时时间（秒）
timeout_seconds = 60
# 调试模式
debug = false

# Claude Provider 配置
[providers.claude]
type = "claude"
# API Key (也可以通过 ANTHROPIC_API_KEY 环境变量设置)
api_key = "your-anthropic-api-key-here"
# API 基础 URL
base_url = "https://api.anthropic.com"
# 默认模型
default_model = "claude-sonnet-4-20250514"
# 最大生成 token 数
max_tokens = 8192
# 温度 (0.0 - 1.0)
temperature = 0.7

# OpenAI Provider 配置
[providers.openai]
type = "openai"
# API Key (也可以通过 OPENAI_API_KEY 环境变量设置)
api_key = "your-openai-api-key-here"
# API 基础 URL
base_url = "https://api.openai.com"
# 默认模型
default_model = "gpt-4o"
# 最大生成 token 数
max_tokens = 8192
# 温度 (0.0 - 1.0)
temperature = 0.7

# Mixin Provider 配置 (组合多个 provider)
[providers.mixin]
type = "mixin"
# 策略: fallback (故障转移), round_robin (轮询), weighted_round_robin (加权轮询)
strategy = "fallback"
# 包含的 provider 列表
providers = ["claude", "openai"]
"#.to_string()
}

/// 确保配置文件存在，如果不存在则创建示例配置
pub async fn ensure_config_exists(config_path: &Path) -> std::io::Result<()> {
    if !config_path.exists() {
        // 创建配置目录
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 写入示例配置
        let example_config = create_example_config();
        tokio::fs::write(config_path, example_config).await?;

        println!("已创建示例配置文件: {}", config_path.display());
        println!("请编辑配置文件，设置你的 API Key");
    }
    Ok(())
}

/// 处理 config 子命令
pub async fn handle_config_command(
    command: ConfigCommands,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // 确保配置文件存在
    ensure_config_exists(config_path).await?;

    // 加载配置
    let config_manager = ConfigManager::new();
    config_manager.load_from_file(config_path).await?;

    match command {
        ConfigCommands::List => {
            let providers = config_manager.get_provider_names().await;
            let default_provider = config_manager
                .get_default_provider()
                .await
                .map(|(name, _)| name);

            if providers.is_empty() {
                println!("没有配置任何 provider");
            } else {
                println!("已配置的 providers:");
                for name in providers {
                    let marker = if Some(name.clone()) == default_provider {
                        " (默认)"
                    } else {
                        ""
                    };
                    println!("  - {}{}", name, marker);
                }
            }
        }

        ConfigCommands::Show => {
            let config = config_manager.get_config().await;
            println!("当前配置:");
            println!("  默认 provider: {}", config.global.default_provider);
            println!("  日志级别: {}", config.global.log_level);
            println!("  超时时间: {} 秒", config.global.timeout_seconds);
            println!("  调试模式: {}", config.global.debug);
            println!();
            println!("Providers:");
            for (name, provider) in &config.providers {
                let kind = match provider {
                    ProviderConfig::Claude(_) => "claude",
                    ProviderConfig::OpenAi(_) => "openai",
                    ProviderConfig::Mixin(_) => "mixin",
                };
                println!("  - {} (type: {})", name, kind);
            }
        }

        ConfigCommands::Switch { name } => {
            let providers = config_manager.get_provider_names().await;
            if !providers.contains(&name) {
                eprintln!("错误: provider '{}' 不存在", name);
                eprintln!("可用的 providers: {:?}", providers);
                std::process::exit(1);
            }

            // 更新默认 provider
            config_manager
                .update_config(|config| {
                    config.global.default_provider = name.clone();
                })
                .await;

            println!("已切换到 provider: {}", name);
            println!("注意: 此更改仅在内存中，不会保存到配置文件");
        }

        ConfigCommands::Test { name } => {
            let provider_name = match name {
                Some(n) => n,
                None => {
                    match config_manager.get_default_provider().await {
                        Some((name, _)) => name,
                        None => {
                            eprintln!("错误: 没有指定 provider，且没有配置默认 provider");
                            std::process::exit(1);
                        }
                    }
                }
            };

            let provider = config_manager.get_provider(&provider_name).await;
            match provider {
                Some(p) => {
                    println!("测试 provider: {}", provider_name);
                    match p {
                        ProviderConfig::Claude(config) => {
                            let provider =
                                ClaudeProvider::new(&provider_name, config);
                            if provider.is_available().await {
                                println!("  状态: 可用");
                                println!("  模型: {}", provider.default_model());
                            } else {
                                println!("  状态: 不可用 (未设置 API Key)");
                            }
                        }
                        ProviderConfig::OpenAi(config) => {
                            let provider =
                                OpenAiProvider::new(&provider_name, config);
                            if provider.is_available().await {
                                println!("  状态: 可用");
                                println!("  模型: {}", provider.default_model());
                            } else {
                                println!("  状态: 不可用 (未设置 API Key)");
                            }
                        }
                        ProviderConfig::Mixin(config) => {
                            println!("  类型: Mixin");
                            println!("  策略: {:?}", config.strategy);
                            println!("  包含 providers: {:?}", config.providers);
                        }
                    }
                }
                None => {
                    eprintln!("错误: provider '{}' 不存在", provider_name);
                    std::process::exit(1);
                }
            }
        }

        ConfigCommands::Reload => {
            match config_manager.reload().await {
                Ok(_) => {
                    println!("配置已重新加载");
                }
                Err(e) => {
                    eprintln!("重新加载配置失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
