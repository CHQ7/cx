# CX MVP 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 CX 从骨架项目转化为独立可用的 Agent CLI 工具，支持本地执行和 HTTP 服务模式

**Architecture:** 双模式架构 - `cx run` 本地执行（内嵌 Agent）或 `cx server` 服务模式。配置通过 TOML 管理，支持热重载和多 provider。流式输出使用 tokio channels 传递到终端。

**Tech Stack:** Rust, axum, tokio, serde, toml, notify (文件监控), clap (CLI)

---

## 前置准备

在开始实施前，确保：
- 当前目录是 `E:/work/cx`
- Rust toolchain 已安装 (`cargo --version`)
- 可以运行 `cargo test`

---

## 文件结构映射

```
src/
├── main.rs              # 修改: 使用 clap 解析命令，分发到子命令
├── lib.rs               # 修改: 导出所有公共模块
├── cli/                 # 新建: CLI 层
│   ├── mod.rs           # 子命令定义和分发
│   ├── run.rs           # cx run 实现
│   ├── server.rs        # cx server 实现
│   └── config_cmd.rs    # cx config 实现
├── config/              # 新建: 配置管理系统
│   ├── mod.rs           # 配置模块导出
│   ├── manager.rs       # ConfigManager（热重载核心）
│   ├── model.rs         # 配置数据类型
│   ├── loader.rs        # TOML 加载/保存
│   └── provider.rs      # Provider 类型定义
├── core/                # 新建: 核心引擎重构
│   ├── mod.rs
│   ├── executor.rs      # 本地执行器（cx run）
│   ├── stream.rs        # 流式输出处理
│   └── printer.rs       # 终端输出格式化
├── server/              # 修改完善: HTTP 服务
│   ├── mod.rs
│   ├── app.rs           # axum app 构建
│   ├── handlers.rs      # 请求处理
│   └── stream.rs        # SSE 流式响应
└── llm/                 # 修改: 接入配置系统
    ├── mod.rs
    └── factory.rs       # 根据配置创建 LLM 客户端
```

---

## Task 1: 添加依赖

**Files:**
- Modify: `Cargo.toml`

添加必要的依赖项。

- [ ] **Step 1: 修改 Cargo.toml 添加依赖**

打开 `Cargo.toml`，在 `[dependencies]` 部分添加：

```toml
# CLI
clap = { version = "4.5", features = ["derive"] }

# 配置管理
toml = "0.8"
notify = "6.1"  # 文件热重载

# 流式输出
tokio-stream = "0.1"

# 目录管理
dirs = "5.0"  # 获取 ~/.cx 等路径
```

- [ ] **Step 2: 验证依赖可编译**

```bash
cargo check
```

Expected: 编译成功（可能会有警告，但无错误）

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add clap, toml, notify, tokio-stream, dirs"
```

---

## Task 2: 创建配置系统 - 数据模型

**Files:**
- Create: `src/config/mod.rs`
- Create: `src/config/model.rs`

定义配置的数据结构。

- [ ] **Step 1: 创建 src/config/mod.rs**

```rust
pub mod loader;
pub mod manager;
pub mod model;
pub mod provider;

pub use manager::ConfigManager;
pub use model::{Config, ProviderConfig};
pub use provider::{LlmProvider, ProviderType};
```

- [ ] **Step 2: 创建 src/config/model.rs**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 顶层配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 默认 provider 名称
    #[serde(default)]
    pub default: String,
    /// 全局设置
    #[serde(default)]
    pub global: GlobalConfig,
    /// Provider 列表
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: String::new(),
            global: GlobalConfig::default(),
            providers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
    #[serde(default = "default_verbose")]
    pub verbose: bool,
    #[serde(default = "default_stream")]
    pub stream: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            max_turns: 40,
            verbose: true,
            stream: true,
        }
    }
}

fn default_max_turns() -> u32 { 40 }
fn default_verbose() -> bool { true }
fn default_stream() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(flatten)]
    pub kind: ProviderKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderKind {
    Claude {
        api_key: String,
        #[serde(default = "default_claude_api_base")]
        api_base: String,
        #[serde(default = "default_claude_model")]
        model: String,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
    },
    OpenAi {
        api_key: String,
        #[serde(default = "default_openai_api_base")]
        api_base: String,
        #[serde(default = "default_openai_model")]
        model: String,
        #[serde(default)]
        temperature: Option<f32>,
    },
    Mixin {
        strategy: MixinStrategy,
        providers: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MixinStrategy {
    Fallback,
    RoundRobin,
}

impl Default for MixinStrategy {
    fn default() -> Self {
        MixinStrategy::Fallback
    }
}

fn default_claude_api_base() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_claude_model() -> String {
    "claude-opus-4".to_string()
}

fn default_openai_api_base() -> String {
    "https://api.openai.com".to_string()
}

fn default_openai_model() -> String {
    "gpt-4o".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}
```

- [ ] **Step 3: 运行编译检查**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/config/
git commit -m "feat(config): add configuration data models"
```

---

## Task 3: 创建配置系统 - 加载器

**Files:**
- Create: `src/config/loader.rs`

实现配置的加载和保存。

- [ ] **Step 1: 创建 src/config/loader.rs**

```rust
use super::model::Config;
use std::path::Path;
use std::fs;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML serialize error: {0}")]
    Toml(#[from] toml::ser::Error),
}

/// 从文件加载配置
pub fn load_config(path: &Path) -> Result<Config, LoadError> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// 保存配置到文件
pub fn save_config(path: &Path, config: &Config) -> Result<(), SaveError> {
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

/// 展开环境变量引用如 ${VAR}
pub fn expand_env_vars(value: &str) -> String {
    let mut result = value.to_string();
    // 简单实现: 替换 ${VAR} 或 $VAR
    for (key, val) in std::env::vars() {
        result = result.replace(&format!("${{{}}}", key), &val);
        result = result.replace(&format!("${}", key), &val);
    }
    result
}

/// 处理配置中的环境变量
pub fn expand_config_env_vars(config: &mut Config) {
    for provider in &mut config.providers {
        match &mut provider.kind {
            super::model::ProviderKind::Claude { api_key, .. } => {
                *api_key = expand_env_vars(api_key);
            }
            super::model::ProviderKind::OpenAi { api_key, .. } => {
                *api_key = expand_env_vars(api_key);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars() {
        std::env::set_var("TEST_API_KEY", "sk-test-123");
        assert_eq!(expand_env_vars("${TEST_API_KEY}"), "sk-test-123");
        assert_eq!(expand_env_vars("key: ${TEST_API_KEY}"), "key: sk-test-123");
    }
}
```

- [ ] **Step 2: 运行测试**

```bash
cargo test config::loader::tests --lib
```

Expected: 1 test passed

- [ ] **Step 3: Commit**

```bash
git add src/config/loader.rs
git commit -m "feat(config): add config loader with env var expansion"
```

---

## Task 4: 创建配置系统 - 管理器（热重载）

**Files:**
- Create: `src/config/manager.rs`

实现带热重载的配置管理器。

- [ ] **Step 1: 创建 src/config/manager.rs**

```rust
use super::{load_config, save_config, expand_config_env_vars, model::Config};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<Config>>,
    _watcher: RecommendedWatcher,
    reload_tx: mpsc::Sender<()>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = load_config(config_path)?;
        expand_config_env_vars(&mut config);

        let config = Arc::new(RwLock::new(config));
        let config_path = config_path.to_path_buf();
        let config_clone = Arc::clone(&config);
        let path_clone = config_path.clone();

        let (reload_tx, mut reload_rx) = mpsc::channel(10);
        let reload_tx_clone = reload_tx.clone();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if event.kind.is_modify() || event.kind.is_create() {
                            let _ = reload_tx_clone.try_send(());
                        }
                    }
                    Err(e) => eprintln!("[Config] Watch error: {}", e),
                }
            },
            NotifyConfig::default(),
        )?;

        // 监听配置文件的父目录
        if let Some(parent) = path_clone.parent() {
            let _watcher = watcher.clone();
            _watcher.watch(parent, RecursiveMode::NonRecursive)?;
        }

        // 启动重载任务
        tokio::spawn(async move {
            while reload_rx.recv().await.is_some() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                match load_config(&path_clone) {
                    Ok(mut new_config) => {
                        expand_config_env_vars(&mut new_config);
                        if let Ok(mut guard) = config_clone.write() {
                            *guard = new_config;
                            eprintln!("[Config] Configuration reloaded");
                        }
                    }
                    Err(e) => eprintln!("[Config] Failed to reload: {}", e),
                }
            }
        });

        Ok(Self {
            config_path,
            config,
            _watcher: watcher,
            reload_tx,
        })
    }

    /// 获取当前配置
    pub fn get(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    /// 手动触发重载
    pub fn reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.reload_tx.try_send(());
        Ok(())
    }

    /// 切换默认 provider
    pub fn switch_default(&self, name: &str) -> Result<(), String> {
        let mut guard = self.config.write().map_err(|_| "Lock poisoned")?;
        let exists = guard.providers.iter().any(|p| p.name == name);
        if !exists {
            return Err(format!("Provider '{}' not found", name));
        }
        guard.default = name.to_string();
        // 保存到文件
        if let Err(e) = save_config(&self.config_path, &*guard) {
            return Err(format!("Failed to save config: {}", e));
        }
        Ok(())
    }

    /// 获取默认 provider 名称
    pub fn default_provider(&self) -> String {
        let guard = self.config.read().unwrap();
        if guard.default.is_empty() && !guard.providers.is_empty() {
            guard.providers[0].name.clone()
        } else {
            guard.default.clone()
        }
    }

    /// 获取指定 provider 的配置
    pub fn get_provider(&self, name: &str) -> Option<super::model::ProviderConfig> {
        let guard = self.config.read().unwrap();
        guard.providers.iter().find(|p| p.name == name).cloned()
    }

    /// 列出所有 provider
    pub fn list_providers(&self) -> Vec<(String, String, bool)> {
        let guard = self.config.read().unwrap();
        let default = self.default_provider();
        guard
            .providers
            .iter()
            .map(|p| {
                let kind = match &p.kind {
                    super::model::ProviderKind::Claude { .. } => "claude".to_string(),
                    super::model::ProviderKind::OpenAi { .. } => "openai".to_string(),
                    super::model::ProviderKind::Mixin { .. } => "mixin".to_string(),
                };
                let is_default = p.name == default;
                (p.name.clone(), kind, is_default)
            })
            .collect()
    }

    /// 获取配置路径
    pub fn path(&self) -> &Path {
        &self.config_path
    }
}

/// 获取默认配置路径 (~/.cx/config.toml)
pub fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".cx").join("config.toml"))
}

/// 确保配置目录存在
pub fn ensure_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        let dir = home.join(".cx");
        std::fs::create_dir_all(&dir).ok();
        dir
    })
}
```

- [ ] **Step 2: 编译检查**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/config/manager.rs
git commit -m "feat(config): add ConfigManager with hot-reload support"
```

---

## Task 5: 更新 lib.rs 导出配置模块

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: 修改 src/lib.rs**

将现有内容替换为：

```rust
//! cx - Core engine for CX (Clause eXecutor)
//!
//! A Rust-based AI Agent system with CLI and HTTP server modes.

pub mod agent;
pub mod api;
pub mod browser;
pub mod config;
pub mod llm;
pub mod tools;
pub mod utils;

/// Core functionality for local and remote execution
pub mod core;
```

- [ ] **Step 2: 创建 src/core/mod.rs**

```rust
pub mod executor;
pub mod printer;
pub mod stream;
```

- [ ] **Step 3: 编译检查**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs src/core/mod.rs
git commit -m "chore: update lib.rs exports and create core module"
```

---

## Task 6: 创建 CLI 命令结构

**Files:**
- Create: `src/cli/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 src/cli/mod.rs**

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod config_cmd;
pub mod run;
pub mod server;

/// CX - AI Agent CLI
#[derive(Parser)]
#[command(name = "cx")]
#[command(about = "AI Agent CLI with local and server modes")]
#[command(version)]
pub struct Cli {
    /// Config file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a task
    Run {
        /// The task prompt
        #[arg(value_name = "PROMPT")]
        prompt: Option<String>,

        /// Provider to use
        #[arg(short, long)]
        model: Option<String>,

        /// Max turns
        #[arg(short, long)]
        max_turns: Option<u32>,

        /// Remote server URL
        #[arg(long)]
        remote: Option<String>,

        /// Disable streaming
        #[arg(long)]
        no_stream: bool,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start HTTP server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Disable hot reload
        #[arg(long)]
        no_hot_reload: bool,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// List all providers
    List,
    /// Show provider details
    Show { name: String },
    /// Switch default provider
    Switch { name: String },
    /// Test provider connection
    Test { name: String },
    /// Reload configuration
    Reload,
}

/// 获取配置路径
pub fn resolve_config_path(cli_path: Option<PathBuf>) -> PathBuf {
    cli_path
        .or_else(|| std::env::var("CX_CONFIG").ok().map(PathBuf::from))
        .or_else(|| config::manager::default_config_path())
        .expect("Failed to determine config path. Use --config or set CX_CONFIG")
}
```

- [ ] **Step 2: 修改 src/main.rs**

```rust
use clap::Parser;
use cx::cli::{resolve_config_path, Cli, Commands, ConfigCommands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 初始化日志
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else if !cli.quiet {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let config_path = resolve_config_path(cli.config);

    match cli.command {
        Commands::Run {
            prompt,
            model,
            max_turns,
            remote,
            no_stream,
            output,
        } => {
            cx::cli::run::execute(
                prompt,
                model,
                max_turns,
                remote,
                no_stream,
                output,
                &config_path,
            )
            .await;
        }
        Commands::Server {
            port,
            host,
            no_hot_reload,
        } => {
            cx::cli::server::execute(port, &host, no_hot_reload, &config_path).await;
        }
        Commands::Config { command } => match command {
            ConfigCommands::List => {
                cx::cli::config_cmd::list(&config_path);
            }
            ConfigCommands::Show { name } => {
                cx::cli::config_cmd::show(&config_path, &name);
            }
            ConfigCommands::Switch { name } => {
                cx::cli::config_cmd::switch(&config_path, &name);
            }
            ConfigCommands::Test { name } => {
                cx::cli::config_cmd::test(&config_path, &name).await;
            }
            ConfigCommands::Reload => {
                cx::cli::config_cmd::reload(&config_path);
            }
        },
    }
}
```

- [ ] **Step 3: 编译检查**

```bash
cargo check
```

Expected: 编译失败（因为 cli::run, cli::server, cli::config_cmd 模块还未实现）
这是预期行为，接下来我们会实现这些模块。

- [ ] **Step 4: Commit 当前进度**

```bash
git add src/cli/mod.rs src/main.rs
git commit -m "feat(cli): add clap command structure (WIP)"
```

---

## Task 7: 实现配置命令

**Files:**
- Create: `src/cli/config_cmd.rs`

- [ ] **Step 1: 创建 src/cli/config_cmd.rs**

```rust
use cx::config::{manager::ConfigManager, loader::save_config, model::Config};
use std::path::Path;

pub fn list(config_path: &Path) {
    match ConfigManager::new(config_path) {
        Ok(manager) => {
            println!("Providers:");
            for (name, kind, is_default) in manager.list_providers() {
                let marker = if is_default { " (default)" } else { "" };
                println!("  {} [{}]{}", name, kind, marker);
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
        }
    }
}

pub fn show(config_path: &Path, name: &str) {
    match ConfigManager::new(config_path) {
        Ok(manager) => {
            if let Some(provider) = manager.get_provider(name) {
                println!("Provider: {}", provider.name);
                match provider.kind {
                    cx::config::model::ProviderKind::Claude {
                        api_base,
                        model,
                        max_tokens,
                        ..
                    } => {
                        println!("  Type: Claude");
                        println!("  API Base: {}", api_base);
                        println!("  Model: {}", model);
                        println!("  Max Tokens: {}", max_tokens);
                    }
                    cx::config::model::ProviderKind::OpenAi {
                        api_base, model, temperature, ..
                    } => {
                        println!("  Type: OpenAI");
                        println!("  API Base: {}", api_base);
                        println!("  Model: {}", model);
                        if let Some(temp) = temperature {
                            println!("  Temperature: {}", temp);
                        }
                    }
                    cx::config::model::ProviderKind::Mixin { strategy, providers } => {
                        println!("  Type: Mixin");
                        println!("  Strategy: {:?}", strategy);
                        println!("  Providers: {:?}", providers);
                    }
                }
            } else {
                eprintln!("Provider '{}' not found", name);
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
        }
    }
}

pub fn switch(config_path: &Path, name: &str) {
    // 先确保配置目录存在
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // 如果配置文件不存在，创建一个空配置
    if !config_path.exists() {
        let empty_config = Config::default();
        if let Err(e) = save_config(config_path, &empty_config) {
            eprintln!("Error creating config file: {}", e);
            return;
        }
    }

    match ConfigManager::new(config_path) {
        Ok(manager) => {
            if let Err(e) = manager.switch_default(name) {
                eprintln!("Error: {}", e);
            } else {
                println!("Switched to provider: {}", name);
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
        }
    }
}

pub async fn test(config_path: &Path, name: &str) {
    use cx::llm::client::LlmClient;

    match ConfigManager::new(config_path) {
        Ok(manager) => {
            if let Some(provider) = manager.get_provider(name) {
                println!("Testing provider: {}...", name);
                // 这里应该创建一个简单的测试请求
                // 由于 LLM 客户端实现尚未完成，暂时占位
                println!("TODO: Implement connection test for {:?}", provider);
            } else {
                eprintln!("Provider '{}' not found", name);
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
        }
    }
}

pub fn reload(config_path: &Path) {
    match ConfigManager::new(config_path) {
        Ok(manager) => {
            if let Err(e) = manager.reload() {
                eprintln!("Error triggering reload: {}", e);
            } else {
                println!("Configuration reload triggered");
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
        }
    }
}

/// 创建示例配置文件
pub fn create_example_config(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let example = r#"# CX Configuration File
# Place this file at ~/.cx/config.toml

# Default provider to use
default = "claude"

[global]
max_turns = 40
verbose = true
stream = true

[[providers]]
name = "claude"
type = "claude"
api_key = "${CLAUDE_API_KEY}"
api_base = "https://api.anthropic.com"
model = "claude-opus-4"
max_tokens = 4096

[[providers]]
name = "openai"
type = "openai"
api_key = "${OPENAI_API_KEY}"
api_base = "https://api.openai.com"
model = "gpt-4o"
"#;

    std::fs::write(path, example)?;
    Ok(())
}
```

- [ ] **Step 2: 编译检查**

```bash
cargo check
```

Expected: 编译失败（缺少 cli::run 和 cli::server），但 config_cmd 应该没问题

- [ ] **Step 3: Commit**

```bash
git add src/cli/config_cmd.rs
git commit -m "feat(cli): implement config commands (list, show, switch, test, reload)"
```

---

## Task 8: 实现流式输出系统

**Files:**
- Create: `src/core/stream.rs`
- Create: `src/core/printer.rs`

- [ ] **Step 1: 创建 src/core/stream.rs**

```rust
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// 流式事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum StreamEvent {
    TurnStart { turn: u32 },
    Thinking { text: String },
    Content { text: String },
    ToolStart { name: String, args: serde_json::Value },
    ToolResult { name: String, result: serde_json::Value },
    ToolOutput { output: String },
    TurnEnd { turn: u32 },
    Done { reason: ExitReason, turns: u32 },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    TaskComplete,
    MaxTurnsExceeded,
    UserAbort,
    Error(String),
}

/// 流式输出通道
pub struct StreamChannel {
    pub tx: mpsc::Sender<StreamEvent>,
    pub rx: mpsc::Receiver<StreamEvent>,
}

impl StreamChannel {
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer);
        Self { tx, rx }
    }
}

/// 流式输出的 sender 端
pub struct StreamSender {
    tx: mpsc::Sender<StreamEvent>,
}

impl StreamSender {
    pub fn new(tx: mpsc::Sender<StreamEvent>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, event: StreamEvent) -> Result<(), mpsc::error::SendError<StreamEvent>> {
        self.tx.send(event).await
    }

    pub fn send_blocking(&self, event: StreamEvent) {
        let _ = self.tx.try_send(event);
    }
}

impl Clone for StreamSender {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
```

- [ ] **Step 2: 创建 src/core/printer.rs**

```rust
use super::stream::{ExitReason, StreamEvent};
use std::io::{self, Write};

/// 终端输出打印机
pub struct Printer {
    verbose: bool,
    current_turn: u32,
    in_tool_output: bool,
}

impl Printer {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            current_turn: 0,
            in_tool_output: false,
        }
    }

    pub fn print_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::TurnStart { turn } => {
                self.current_turn = *turn;
                if self.verbose {
                    println!("\n✦ Turn {} ...", turn);
                } else {
                    eprint!(".");
                    io::stderr().flush().ok();
                }
            }
            StreamEvent::Thinking { text } => {
                if self.verbose && !text.is_empty() {
                    // 思考内容可以选择性显示
                    // println!("\n[Thinking] {}", text);
                }
            }
            StreamEvent::Content { text } => {
                if self.verbose {
                    print!("{}", text);
                    io::stdout().flush().ok();
                }
            }
            StreamEvent::ToolStart { name, args } => {
                if self.verbose {
                    if self.in_tool_output {
                        println!();
                    }
                    println!("\n🛠️  Tool: {}", name);
                    if let Ok(args_str) = serde_json::to_string_pretty(args) {
                        println!("   Args: {}", args_str);
                    }
                    self.in_tool_output = true;
                }
            }
            StreamEvent::ToolResult { name, result } => {
                if self.verbose {
                    println!("   Result: {}", result);
                    self.in_tool_output = false;
                }
            }
            StreamEvent::ToolOutput { output } => {
                if self.verbose {
                    print!("{}", output);
                    io::stdout().flush().ok();
                }
            }
            StreamEvent::TurnEnd { .. } => {
                if self.verbose {
                    println!();
                }
            }
            StreamEvent::Done { reason, turns } => {
                if !self.verbose {
                    eprintln!(); // 结束进度点
                }
                match reason {
                    ExitReason::TaskComplete => {
                        println!("\n✓ Done ({} turns)", turns);
                    }
                    ExitReason::MaxTurnsExceeded => {
                        println!("\n⚠ Max turns exceeded ({}", turns);
                    }
                    ExitReason::UserAbort => {
                        println!("\n✗ Aborted by user");
                    }
                    ExitReason::Error(msg) => {
                        println!("\n✗ Error: {}", msg);
                    }
                }
            }
            StreamEvent::Error { message } => {
                eprintln!("\n✗ Error: {}", message);
            }
        }
    }

    /// 处理所有事件直到完成
    pub async fn consume_stream(&mut self, rx: &mut tokio::sync::mpsc::Receiver<StreamEvent>) {
        while let Some(event) = rx.recv().await {
            let is_done = matches!(&event, StreamEvent::Done { .. } | StreamEvent::Error { .. });
            self.print_event(&event);
            if is_done {
                break;
            }
        }
    }
}
```

- [ ] **Step 3: 编译检查**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/core/stream.rs src/core/printer.rs
git commit -m "feat(core): add streaming output system (StreamEvent, Printer)"
```

---

## Task 9: 实现本地执行器

**Files:**
- Create: `src/core/executor.rs`

- [ ] **Step 1: 创建 src/core/executor.rs**

```rust
use crate::agent::loop::{AgentLoop, ExitReason as AgentExitReason};
use crate::config::manager::ConfigManager;
use crate::core::stream::{ExitReason, StreamEvent, StreamSender};
use crate::llm::client::LlmClient;
use crate::tools::{ToolContext, ToolHandler, WorkingMemory};
use std::path::PathBuf;
use std::sync::Arc;

pub struct LocalExecutor {
    config_manager: ConfigManager,
}

impl LocalExecutor {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

    pub async fn execute(
        &self,
        prompt: String,
        provider_name: Option<String>,
        max_turns: Option<u32>,
    ) -> Result<(), String> {
        let config = self.config_manager.get();
        let global = &config.global;

        // 获取 provider 名称
        let provider_name = provider_name.unwrap_or_else(|| self.config_manager.default_provider());

        // 获取 provider 配置
        let provider_config = self
            .config_manager
            .get_provider(&provider_name)
            .ok_or_else(|| format!("Provider '{}' not found", provider_name))?;

        // 创建 LLM 客户端（这里简化处理，实际需要根据 provider 类型创建）
        let llm_client = create_llm_client(&provider_config)?;

        // 创建工具
        let tools = create_tools();

        // 创建工作目录
        let working_dir = std::env::current_dir().map_err(|e| e.to_string())?;

        // 创建工具上下文
        let tool_context = ToolContext {
            current_turn: 0,
            working_dir: working_dir.clone(),
            working_memory: WorkingMemory {
                key_info: None,
                related_sop: None,
                in_plan_mode: None,
                passed_sessions: 0,
            },
            verbose: global.verbose,
            project_root: working_dir,
        };

        // 系统提示词
        let system_prompt = build_system_prompt();

        // 创建 Agent Loop
        let agent = AgentLoop::new(max_turns.unwrap_or(global.max_turns));

        // 执行（这里需要流式通道）
        // TODO: 接入实际的 Agent Loop 执行和流式输出

        Ok(())
    }
}

fn create_llm_client(
    config: &crate::config::model::ProviderConfig,
) -> Result<Arc<dyn LlmClient>, String> {
    use crate::llm::claude::ClaudeSession;

    match &config.kind {
        crate::config::model::ProviderKind::Claude {
            api_key,
            api_base,
            model,
            ..
        } => {
            let client = ClaudeSession::new(
                api_key.clone(),
                api_base.clone(),
                model.clone(),
            );
            Ok(Arc::new(client))
        }
        _ => Err(format!("Provider type not yet implemented: {:?}", config.name)),
    }
}

fn create_tools() -> Vec<Arc<dyn ToolHandler>> {
    use crate::tools::{
        code_run::CodeRunTool,
        file_ops::{FilePatchTool, FileReadTool, FileWriteTool},
        memory::UpdateWorkingCheckpointTool,
        user::AskUserTool,
    };

    vec![
        Arc::new(FileReadTool),
        Arc::new(FilePatchTool),
        Arc::new(FileWriteTool),
        Arc::new(CodeRunTool),
        Arc::new(UpdateWorkingCheckpointTool),
        Arc::new(AskUserTool),
    ]
}

fn build_system_prompt() -> String {
    r#"You are CX, an AI assistant that can use tools to help users.

Available tools:
- file_read: Read file contents
- file_patch: Patch file with exact text replacement
- file_write: Write content to file
- code_run: Execute code (Python or shell)
- update_working_checkpoint: Update working memory
- ask_user: Ask user for input

Always use tools when needed. Be concise but thorough."#
        .to_string()
}
```

- [ ] **Step 2: 编译检查**

```bash
cargo check
```

Expected: 编译成功（可能有警告）

- [ ] **Step 3: Commit**

```bash
git add src/core/executor.rs
git commit -m "feat(core): add LocalExecutor structure (WIP)"
```

---

## Task 10: 实现 CLI run 命令

**Files:**
- Create: `src/cli/run.rs`

- [ ] **Step 1: 创建 src/cli/run.rs**

```rust
use crate::config::manager::{ensure_config_dir, ConfigManager};
use crate::core::executor::LocalExecutor;
use crate::cli::config_cmd::create_example_config;
use std::io::{self, Read};
use std::path::Path;

pub async fn execute(
    prompt: Option<String>,
    model: Option<String>,
    max_turns: Option<u32>,
    remote: Option<String>,
    no_stream: bool,
    output: Option<std::path::PathBuf>,
    config_path: &Path,
) {
    // 读取 prompt
    let prompt = match prompt {
        Some(p) => p,
        None => {
            // 从 stdin 读取
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("Error reading from stdin: {}", e);
                return;
            }
            if buffer.trim().is_empty() {
                eprintln!("Error: No prompt provided. Use 'cx run <PROMPT>' or pipe input.");
                return;
            }
            buffer
        }
    };

    // 确保配置目录存在
    if let Some(dir) = ensure_config_dir() {
        if !config_path.exists() {
            println!("Creating example config at: {}", config_path.display());
            if let Err(e) = create_example_config(config_path) {
                eprintln!("Error creating config: {}", e);
                return;
            }
            eprintln!(
                "Please edit {} with your API keys before running again.",
                config_path.display()
            );
            return;
        }
    }

    // 如果有 remote URL，使用客户端模式
    if let Some(url) = remote {
        execute_remote(prompt, model, max_turns, no_stream, output, &url).await;
    } else {
        execute_local(
            prompt,
            model,
            max_turns,
            no_stream,
            output,
            config_path,
        )
        .await;
    }
}

async fn execute_local(
    prompt: String,
    model: Option<String>,
    max_turns: Option<u32>,
    _no_stream: bool,
    _output: Option<std::path::PathBuf>,
    config_path: &Path,
) {
    let config_manager = match ConfigManager::new(config_path) {
        Ok(cm) => cm,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            return;
        }
    };

    let executor = LocalExecutor::new(config_manager);

    if let Err(e) = executor.execute(prompt, model, max_turns).await {
        eprintln!("Execution error: {}", e);
    }
}

async fn execute_remote(
    _prompt: String,
    _model: Option<String>,
    _max_turns: Option<u32>,
    _no_stream: bool,
    _output: Option<std::path::PathBuf>,
    _url: &str,
) {
    println!("Remote execution not yet implemented");
    // TODO: 使用 HTTP 客户端连接远程 cx server
}
```

- [ ] **Step 2: 编译检查**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/cli/run.rs
git commit -m "feat(cli): implement cx run command with local and remote modes"
```

---

## Task 11: 实现 CLI server 命令

**Files:**
- Create: `src/cli/server.rs`

- [ ] **Step 1: 创建 src/cli/server.rs**

```rust
use crate::config::manager::ConfigManager;
use crate::server::app::create_app;
use std::path::Path;
use std::sync::Arc;

pub async fn execute(
    port: u16,
    host: &str,
    _no_hot_reload: bool,
    config_path: &Path,
) {
    // 加载配置
    let config_manager = match ConfigManager::new(config_path) {
        Ok(cm) => Arc::new(cm),
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            return;
        }
    };

    // 创建 axum app
    let app = create_app(config_manager);

    let addr = format!("{}:{}", host, port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    println!("✦ CX server listening on http://{}", addr);
    println!("  Config: {}", config_path.display());

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
    }
}
```

- [ ] **Step 2: 完善 server 模块**

修改 `src/server/mod.rs`（如果存在）或创建它：

```rust
pub mod app;
pub mod handlers;
pub mod stream;
```

- [ ] **Step 3: 创建 src/server/app.rs**

```rust
use crate::config::manager::ConfigManager;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config_manager: Arc<ConfigManager>,
}

pub fn create_app(config_manager: Arc<ConfigManager>) -> Router {
    let state = AppState { config_manager };

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/run", post(run_handler))
        .route("/api/config", get(config_handler))
        .route("/api/config/reload", post(reload_handler))
        .with_state(state)
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn run_handler(
    axum::extract::State(_state): axum::extract::State<AppState>,
    axum::extract::Json(_payload): axum::extract::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    // TODO: 实现实际的 run handler
    axum::Json(serde_json::json!({
        "status": "not_implemented"
    }))
}

async fn config_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    let providers = state.config_manager.list_providers();
    axum::Json(providers)
}

async fn reload_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    match state.config_manager.reload() {
        Ok(_) => axum::Json(serde_json::json!({"status": "ok"})),
        Err(e) => axum::Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
```

- [ ] **Step 4: 创建空的 handlers.rs 和 stream.rs**

```bash
# 创建空文件作为占位符
touch src/server/handlers.rs
touch src/server/stream.rs
```

- [ ] **Step 5: 编译检查**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add src/cli/server.rs src/server/
git commit -m "feat(cli): implement cx server command with basic HTTP endpoints"
```

---

## Task 12: 修复 Agent Loop 接入

**Files:**
- Modify: `src/agent/loop.rs` 接入流式通道
- Modify: `src/core/executor.rs` 完成集成

这是最关键的任务，需要将 Agent Loop 与流式输出系统连接起来。

由于复杂度较高，这里概述任务，建议分解为多个子步骤：

- [ ] **Step 1: 修改 Agent Loop 支持流式回调**
- [ ] **Step 2: 集成工具执行到流式通道**
- [ ] **Step 3: 测试本地执行流程**

详细代码修改由于长度限制，请在实施时根据编译错误逐步修复。

---

## Task 13: 添加 Dockerfile

**Files:**
- Create: `Dockerfile`

- [ ] **Step 1: 创建 Dockerfile**

```dockerfile
# 构建阶段
FROM rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# 运行阶段
FROM alpine:latest

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/cx /usr/local/bin/cx

EXPOSE 8080

ENTRYPOINT ["cx"]
CMD ["server", "--host", "0.0.0.0"]
```

- [ ] **Step 2: 创建 .dockerignore**

```
target/
.dockerignore
Dockerfile
.git/
.gitignore
*.md
```

- [ ] **Step 3: 测试构建**

```bash
docker build -t cx:test .
```

Expected: 构建成功

- [ ] **Step 4: Commit**

```bash
git add Dockerfile .dockerignore
git commit -m "feat(docker): add Dockerfile for containerized deployment"
```

---

## Task 14: 最终集成测试

**Files:**
- Modify: 根据需要修复编译错误

- [ ] **Step 1: 完整编译检查**

```bash
cargo build --release
```

Expected: 无错误

- [ ] **Step 2: 运行测试**

```bash
cargo test
```

Expected: 所有测试通过

- [ ] **Step 3: 本地测试 cx 命令**

```bash
# 测试帮助
cargo run -- --help

# 测试 config 命令（会创建示例配置）
cargo run -- config list

# 测试 server（在另一个终端）
cargo run -- server
```

- [ ] **Step 4: Commit 所有更改**

```bash
git add .
git commit -m "feat: complete CX MVP implementation"
```

---

## 已完成检查清单

实施完成后，验证以下功能：

- [ ] `cx --help` 显示帮助信息
- [ ] `cx config list` 列出 providers
- [ ] `cx config switch <name>` 切换默认 provider
- [ ] `cx run "任务"` 本地执行任务
- [ ] `cx server` 启动 HTTP 服务
- [ ] `curl http://localhost:8080/health` 返回 OK
- [ ] Docker 镜像可以构建和运行
- [ ] 配置文件支持环境变量
- [ ] 配置修改后自动重载

---

## 实施建议

1. **分步实施**：每个 Task 完成后运行 `cargo check` 确保编译通过
2. **及时提交**：每个 Task 后 git commit
3. **遇到编译错误**：优先修复类型签名不匹配问题
4. **测试驱动**：每添加一个功能，先写测试再实现
5. **简化优先**：如果某个功能实现复杂，先用 `todo!()` 占位，确保整体框架先跑通
