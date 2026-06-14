use clap::Subcommand;
use std::path::Path;
use crate::config::{ConfigManager, ProviderConfig, LlmProvider};
use crate::config::provider::{ClaudeProvider, OpenAiProvider};

/// 配置管理子命令
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// 初始化配置文件（如果不存在则创建）
    Init,

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
default_provider = "local"
# 日志级别: trace, debug, info, warn, error
log_level = "info"
# 请求超时时间（秒）
timeout_seconds = 60
# 调试模式
debug = false

# =============================================================================
# 本地模型配置 (OpenAI 兼容 API)
# 适用于: Ollama, LM Studio, text-generation-webui, 以及任何 OpenAI 兼容 API
# =============================================================================
[providers.local]
type = "openai"
# API Key (本地模型通常不需要，但某些服务可能需要)
# 可以通过环境变量 OPENAI_API_KEY 设置，或在这里留空/设置占位符
api_key = ""
# 本地 API 基础 URL
# Ollama 默认: http://localhost:11434/v1
# LM Studio 默认: http://localhost:1234/v1
# 自定义服务: http://172.29.16.224:26398/v1
base_url = "http://172.29.16.224:26398/v1"
# 模型名称
# Ollama 示例: llama3.1, qwen2.5, deepseek-coder-v2
# 您的服务: qwen3.6-27b
default_model = "qwen3.6:27b"
# 最大生成 token 数
max_tokens = 4096
# 温度 (0.0 - 1.0)
temperature = 0.7

# =============================================================================
# 易用 AI Claude 配置 (cloud.yiyongai.cn)
# =============================================================================
[providers.yiyong]
type = "claude"
api_key = "sk-Tzb2Cv0rEDiRRPwNiF9RNKDydjeJNstyIKc3BTb1LmLqhhHO"
base_url = "https://cloud.yiyongai.cn"
default_model = "claude-opus-4-8"
max_tokens = 8192
temperature = 0.7

# =============================================================================
# Claude Provider 配置 (Anthropic API)
# =============================================================================
[providers.claude]
type = "claude"
# API Key (也可以通过 ANTHROPIC_API_KEY 环境变量设置)
api_key = ""
# API 基础 URL
base_url = "https://api.anthropic.com"
# 默认模型: claude-opus-4, claude-sonnet-4, claude-haiku-4
default_model = "claude-sonnet-4-20250514"
# 最大生成 token 数
max_tokens = 8192
# 温度 (0.0 - 1.0)
temperature = 0.7

# =============================================================================
# OpenAI Provider 配置
# =============================================================================
[providers.openai]
type = "openai"
# API Key (也可以通过 OPENAI_API_KEY 环境变量设置)
api_key = ""
# API 基础 URL
base_url = "https://api.openai.com"
# 默认模型: gpt-4o, gpt-4-turbo, gpt-3.5-turbo
default_model = "gpt-4o"
# 最大生成 token 数
max_tokens = 8192
# 温度 (0.0 - 1.0)
temperature = 0.7

# =============================================================================
# Mixin Provider 配置 (组合多个 provider，支持故障转移和负载均衡)
# =============================================================================
[providers.backup]
type = "mixin"
# 策略: fallback (故障转移), round_robin (轮询), weighted_round_robin (加权轮询)
strategy = "fallback"
# 包含的 provider 列表（按优先级排序）
providers = ["local", "claude", "openai"]
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
    match command {
        ConfigCommands::Init => {
            // 初始化配置文件
            if config_path.exists() {
                println!("配置文件已存在: {}", config_path.display());
                println!("使用 'cx config show' 查看当前配置");
            } else {
                ensure_config_exists(config_path).await?;
                println!("\n提示: 请编辑配置文件设置您的 API Key:");
                println!("  {}", config_path.display());
                println!("\n本地模型用户: 修改 providers.local 部分");
                println!("Claude 用户: 修改 providers.claude 部分");
                println!("OpenAI 用户: 修改 providers.openai 部分");
            }
            return Ok(());
        }

        _ => {
            // 其他命令需要配置文件存在
            ensure_config_exists(config_path).await?;
        }
    }

    // 加载配置
    let config_manager = ConfigManager::new();
    config_manager.load_from_file(config_path).await?;

    match command {
        ConfigCommands::Init => {
            // 已在上面处理，不会执行到这里
            return Ok(());
        }

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
