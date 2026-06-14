# GenericAgent Rust Core 实现计划

> **日期**: 2026-06-14
> **目标**: 创建 Rust 项目骨架，逐步实现 Agent Loop + 9 个原子工具 + LLM Core
> **技术栈**: Rust, axum, tokio, serde, reqwest

---

## 文件结构

```
ga-core/
├── Cargo.toml              # 项目配置
├── src/
│   ├── main.rs             # HTTP 服务入口
│   ├── lib.rs              # 库入口
│   ├── api/
│   │   ├── mod.rs          # API 路由注册
│   │   ├── routes.rs       # REST endpoints
│   │   └── websocket.rs    # 流式输出 WebSocket
│   ├── agent/
│   │   ├── mod.rs          # Agent 模块导出
│   │   ├── loop.rs         # Agent 主循环
│   │   ├── handler.rs      # GenericAgentHandler
│   │   └── outcome.rs      # StepOutcome, ToolCall, ToolResult
│   ├── llm/
│   │   ├── mod.rs          # LLM 模块导出
│   │   ├── client.rs       # LlmClient trait + LlmError
│   │   ├── claude.rs       # ClaudeSession
│   │   ├── openai.rs       # OpenAiSession
│   │   ├── native.rs       # NativeToolClient
│   │   ├── mixin.rs        # MixinSession
│   │   └── models.rs       # Message, ContentBlock, MockResponse
│   ├── tools/
│   │   ├── mod.rs          # Tools 模块导出 + ToolHandler trait
│   │   ├── code_run.rs     # 代码执行
│   │   ├── file_ops.rs     # 文件读写/补丁
│   │   ├── web.rs          # 浏览器操作
│   │   ├── memory.rs       # 工作记忆/长期记忆
│   │   └── user.rs         # 用户交互
│   ├── browser/
│   │   ├── mod.rs          # Browser 模块导出
│   │   ├── driver.rs       # TMWebDriver
│   │   └── session.rs      # Session 管理
│   └── utils/
│       ├── mod.rs          # Utils 模块导出
│       ├── html.rs         # HTML 简化
│       ├── format.rs       # 字符串格式化
│       └── paths.rs        # 项目路径管理
├── tests/
│   └── integration_tests.rs
└── assets/
    └── tools_schema.json
```

---

## 任务清单

### Task 1: 创建 Rust 项目骨架

**目标**: 初始化 Cargo 项目，配置依赖，建立目录结构

**文件:**
- Create: `ga-core/Cargo.toml`
- Create: `ga-core/src/main.rs`
- Create: `ga-core/src/lib.rs`
- Create: 所有模块的 `mod.rs` 占位文件

- [ ] **Step 1: 创建 Cargo 项目**

```bash
cd E:/work/GA2
cargo new --bin ga-core
cd ga-core
```

- [ ] **Step 2: 编辑 Cargo.toml 添加依赖**

```toml
[package]
name = "ga-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40", features = ["full"] }
axum = { version = "0.7", features = ["ws"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json", "stream"] }
futures = "0.3"
regex = "1.10"
scraper = "0.19"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio-test = "0.4"
```

- [ ] **Step 3: 创建目录结构**

```bash
mkdir -p src/api src/agent src/llm src/tools src/browser src/utils tests assets
```

- [ ] **Step 4: 创建所有 mod.rs 占位文件**

每个模块创建空的 `mod.rs`：

```rust
// src/api/mod.rs
pub mod routes;
pub mod websocket;

// src/agent/mod.rs
pub mod loop;
pub mod handler;
pub mod outcome;

// src/llm/mod.rs
pub mod client;
pub mod claude;
pub mod openai;
pub mod native;
pub mod mixin;
pub mod models;

// src/tools/mod.rs
pub mod code_run;
pub mod file_ops;
pub mod web;
pub mod memory;
pub mod user;

// src/browser/mod.rs
pub mod driver;
pub mod session;

// src/utils/mod.rs
pub mod html;
pub mod format;
pub mod paths;
```

- [ ] **Step 5: 创建 lib.rs**

```rust
pub mod api;
pub mod agent;
pub mod llm;
pub mod tools;
pub mod browser;
pub mod utils;
```

- [ ] **Step 6: 创建 main.rs**

```rust
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(|| async { "OK" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("ga-core listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 7: 编译验证**

```bash
cargo check
```

Expected: 编译成功，无错误

- [ ] **Step 8: 运行验证**

```bash
cargo run &
curl http://localhost:8080/health
```

Expected: 返回 `OK`

- [ ] **Step 9: Commit**

```bash
git add ga-core/
git commit -m "feat(rust): initialize ga-core project skeleton with axum"
```

---

### Task 2: 定义核心数据类型（llm/models.rs + agent/outcome.rs）

**目标**: 实现所有核心数据结构，确保与 Python 版本 JSON 兼容

**文件:**
- Create: `ga-core/src/llm/models.rs`
- Create: `ga-core/src/agent/outcome.rs`
- Modify: `ga-core/src/llm/mod.rs`
- Modify: `ga-core/src/agent/mod.rs`

- [ ] **Step 1: 实现 llm/models.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<ToolResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct MockFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct MockToolCall {
    pub function: MockFunction,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub thinking: String,
    pub content: String,
    pub tool_calls: Vec<MockToolCall>,
    pub raw: String,
    pub stop_reason: String,
}
```

- [ ] **Step 2: 实现 agent/outcome.rs**

```rust
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: Value,
    pub id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub data: Option<Value>,
    pub next_prompt: Option<String>,
    pub should_exit: bool,
}

impl StepOutcome {
    pub fn exit(data: Option<Value>) -> Self {
        Self { data, next_prompt: None, should_exit: true }
    }

    pub fn continue_with(prompt: String, data: Option<Value>) -> Self {
        Self { data, next_prompt: Some(prompt), should_exit: false }
    }

    pub fn done(data: Option<Value>) -> Self {
        Self { data, next_prompt: None, should_exit: false }
    }
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}
```

- [ ] **Step 3: 更新 llm/mod.rs**

```rust
pub mod client;
pub mod claude;
pub mod openai;
pub mod native;
pub mod mixin;
pub mod models;
```

- [ ] **Step 4: 更新 agent/mod.rs**

```rust
pub mod loop;
pub mod handler;
pub mod outcome;
```

- [ ] **Step 5: 编译验证**

```bash
cargo check
```

Expected: 编译成功

- [ ] **Step 6: Commit**

```bash
git add ga-core/src/llm/models.rs ga-core/src/agent/outcome.rs
git add ga-core/src/llm/mod.rs ga-core/src/agent/mod.rs
git commit -m "feat(rust): define core data types (Message, ContentBlock, StepOutcome)"
```

---

### Task 3: 定义 LlmClient trait 和错误类型

**目标**: 定义 LLM 客户端抽象接口

**文件:**
- Create: `ga-core/src/llm/client.rs`

- [ ] **Step 1: 实现 llm/client.rs**

```rust
use async_trait::async_trait;
use futures::stream::BoxStream;
use thiserror::Error;

use super::models::{Message, MockResponse};

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Rate limited")]
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError>;

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<BoxStream<'static, Result<String, LlmError>>, LlmError>;
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add ga-core/src/llm/client.rs
git commit -m "feat(rust): define LlmClient trait and LlmError"
```

---

### Task 4: 实现 ToolHandler trait

**目标**: 定义工具执行接口

**文件:**
- Create: `ga-core/src/tools/mod.rs`

- [ ] **Step 1: 实现 tools/mod.rs**

```rust
use async_trait::async_trait;
use std::path::PathBuf;
use thiserror::Error;

use crate::agent::outcome::StepOutcome;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct WorkingMemory {
    pub key_info: Option<String>,
    pub related_sop: Option<String>,
    pub in_plan_mode: Option<String>,
    pub passed_sessions: u32,
}

#[derive(Debug)]
pub struct ToolContext {
    pub current_turn: u32,
    pub working_dir: PathBuf,
    pub working_memory: WorkingMemory,
    pub verbose: bool,
    pub project_root: PathBuf,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &'static str;

    async fn execute(
        &self,
        args: serde_json::Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError>;
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add ga-core/src/tools/mod.rs
git commit -m "feat(rust): define ToolHandler trait and ToolContext"
```

---

### Task 5: 实现文件操作工具（file_ops.rs）

**目标**: 实现 file_read, file_patch, file_write 三个工具

**文件:**
- Create: `ga-core/src/tools/file_ops.rs`

- [ ] **Step 1: 实现 file_ops.rs**

```rust
use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::agent::outcome::StepOutcome;
use super::{ToolContext, ToolError, ToolHandler, WorkingMemory};

pub struct FileReadTool;
pub struct FilePatchTool;
pub struct FileWriteTool;

#[async_trait]
impl ToolHandler for FileReadTool {
    fn name(&self) -> &'static str { "file_read" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("path required".to_string()))?;

        let path = resolve_path(path_str, &context.working_dir)?;
        let content = fs::read_to_string(&path)
            .map_err(|e| ToolError::Io(e))?;

        let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

        let lines: Vec<&str> = content.lines().collect();
        let end = (start + count - 1).min(lines.len());
        let selected = &lines[start.saturating_sub(1)..end];

        let result = selected.iter().enumerate()
            .map(|(i, line)| format!("{:4} | {}", start + i, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(StepOutcome::done(Some(Value::String(result))))
    }
}

#[async_trait]
impl ToolHandler for FilePatchTool {
    fn name(&self) -> &'static str { "file_patch" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("path required".to_string()))?;
        let old_content = args.get("old_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("old_content required".to_string()))?;
        let new_content = args.get("new_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("new_content required".to_string()))?;

        let path = resolve_path(path_str, &context.working_dir)?;
        let content = fs::read_to_string(&path)
            .map_err(|e| ToolError::Io(e))?;

        let matches: Vec<_> = content.match_indices(old_content).collect();
        if matches.len() != 1 {
            return Err(ToolError::ExecutionFailed(
                format!("Expected 1 match, found {}", matches.len())
            ));
        }

        let new_text = content.replacen(old_content, new_content, 1);
        fs::write(&path, new_text)
            .map_err(|e| ToolError::Io(e))?;

        Ok(StepOutcome::done(Some(Value::String("patched".to_string()))))
    }
}

#[async_trait]
impl ToolHandler for FileWriteTool {
    fn name(&self) -> &'static str { "file_write" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("path required".to_string()))?;
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("content required".to_string()))?;
        let mode = args.get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("overwrite");

        let path = resolve_path(path_str, &context.working_dir)?;

        match mode {
            "overwrite" => fs::write(&path, content),
            "append" => {
                use std::io::Write;
                let mut file = fs::OpenOptions::new().append(true).create(true).open(&path)?;
                file.write_all(content.as_bytes())
            },
            "prepend" => {
                let existing = fs::read_to_string(&path).unwrap_or_default();
                fs::write(&path, format!("{}{}", content, existing))
            },
            _ => return Err(ToolError::InvalidArgs(format!("invalid mode: {}", mode))),
        }.map_err(|e| ToolError::Io(e))?;

        Ok(StepOutcome::done(Some(Value::String("written".to_string()))))
    }
}

fn resolve_path(path_str: &str, working_dir: &Path) -> Result<std::path::PathBuf, ToolError> {
    let path = Path::new(path_str);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(working_dir.join(path))
    }
}
```

- [ ] **Step 2: 添加单元测试**

在 file_ops.rs 底部添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn test_context(dir: &TempDir) -> ToolContext {
        ToolContext {
            current_turn: 1,
            working_dir: dir.path().to_path_buf(),
            working_memory: WorkingMemory { key_info: None, related_sop: None, in_plan_mode: None, passed_sessions: 0 },
            verbose: false,
            project_root: dir.path().to_path_buf(),
        }
    }

    #[tokio::test]
    async fn test_file_read() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line1\nline2\nline3\n").unwrap();

        let tool = FileReadTool;
        let mut ctx = test_context(&dir);
        let args = serde_json::json!({"path": "test.txt", "start": 1, "count": 2});
        let result = tool.execute(args, &mut ctx).await.unwrap();

        let data = result.data.unwrap().as_str().unwrap().to_string();
        assert!(data.contains("line1"));
        assert!(data.contains("line2"));
        assert!(!data.contains("line3"));
    }

    #[tokio::test]
    async fn test_file_patch() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let tool = FilePatchTool;
        let mut ctx = test_context(&dir);
        let args = serde_json::json!({
            "path": "test.txt",
            "old_content": "world",
            "new_content": "rust"
        });
        tool.execute(args, &mut ctx).await.unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello rust");
    }
}
```

- [ ] **Step 3: 添加 tempfile 到 dev-dependencies**

编辑 `Cargo.toml`：

```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.10"
```

- [ ] **Step 4: 编译并测试**

```bash
cargo test tools::file_ops::tests
```

Expected: 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add ga-core/src/tools/file_ops.rs ga-core/Cargo.toml
git commit -m "feat(rust): implement file_ops tools (read, patch, write) with tests"
```

---

### Task 6: 实现代码执行工具（code_run.rs）

**目标**: 实现 code_run 工具（Python/PowerShell 代码执行）

**文件:**
- Create: `ga-core/src/tools/code_run.rs`

- [ ] **Step 1: 实现 code_run.rs**

```rust
use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::agent::outcome::StepOutcome;
use super::{ToolContext, ToolError, ToolHandler};

pub struct CodeRunTool;

#[async_trait]
impl ToolHandler for CodeRunTool {
    fn name(&self) -> &'static str { "code_run" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let script = args.get("script")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let code_type = args.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("python");
        let timeout_secs = args.get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let cwd = args.get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| std::path::PathBuf::from(s))
            .unwrap_or_else(|| context.working_dir.clone());

        let result = match code_type {
            "python" | "py" => run_python(script, &cwd, timeout_secs).await,
            "powershell" | "bash" | "sh" | "shell" | "ps1" | "pwsh" => {
                run_shell(script, code_type, &cwd, timeout_secs).await
            },
            _ => return Err(ToolError::InvalidArgs(format!("unsupported type: {}", code_type))),
        };

        Ok(StepOutcome::done(Some(result?)))
    }
}

async fn run_python(script: &str, cwd: &std::path::Path, timeout_secs: u64) -> Result<Value, ToolError> {
    let output = timeout(
        Duration::from_secs(timeout_secs),
        Command::new("python")
            .args(&["-c", script])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    ).await
    .map_err(|_| ToolError::ExecutionFailed("timeout".to_string()))?
    .map_err(|e| ToolError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(serde_json::json!({
        "status": if output.status.success() { "success" } else { "error" },
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string(),
        "exit_code": output.status.code(),
    }))
}

async fn run_shell(script: &str, shell_type: &str, cwd: &std::path::Path, timeout_secs: u64) -> Result<Value, ToolError> {
    let (cmd, args) = if cfg!(target_os = "windows") {
        match shell_type {
            "powershell" | "ps1" | "pwsh" => {
                let ps = if which::which("pwsh").is_ok() { "pwsh" } else { "powershell" };
                (ps, vec!["-NoProfile", "-NonInteractive", "-Command", script])
            },
            _ => ("cmd", vec!["/C", script]),
        }
    } else {
        ("bash", vec!["-c", script])
    };

    let output = timeout(
        Duration::from_secs(timeout_secs),
        Command::new(cmd)
            .args(&args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    ).await
    .map_err(|_| ToolError::ExecutionFailed("timeout".to_string()))?
    .map_err(|e| ToolError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(serde_json::json!({
        "status": if output.status.success() { "success" } else { "error" },
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string(),
        "exit_code": output.status.code(),
    }))
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add ga-core/src/tools/code_run.rs
git commit -m "feat(rust): implement code_run tool (python/shell execution)"
```

---

### Task 7: 实现 Agent 主循环（agent/loop.rs）

**目标**: 实现 Agent 主循环，连接 LLM 和工具执行

**文件:**
- Create: `ga-core/src/agent/loop.rs`

- [ ] **Step 1: 实现 loop.rs**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::outcome::{StepOutcome, ToolCall, ToolResult};
use crate::llm::client::{LlmClient, ToolSchema};
use crate::llm::models::{ContentBlock, Message, MockResponse, Role};
use crate::tools::{ToolContext, ToolHandler, ToolError};

pub struct AgentLoop {
    pub max_turns: u32,
    pub verbose: bool,
}

impl AgentLoop {
    pub fn new(max_turns: u32) -> Self {
        Self { max_turns, verbose: true }
    }

    pub async fn run(
        &self,
        client: Arc<dyn LlmClient>,
        system_prompt: String,
        user_input: String,
        tools: Vec<Arc<dyn ToolHandler>>,
        tools_schema: Vec<ToolSchema>,
        mut context: ToolContext,
    ) -> Result<RunResult, AgentError> {
        let mut messages = vec![
            Message {
                role: Role::System,
                content: vec![ContentBlock::Text { text: system_prompt }],
                tool_results: None,
            },
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: user_input }],
                tool_results: None,
            },
        ];

        let mut turn = 0u32;

        while turn < self.max_turns {
            turn += 1;
            context.current_turn = turn;

            // Call LLM
            let response = client.chat(messages.clone(), Some(tools_schema.clone())).await
                .map_err(|e| AgentError::LlmError(e.to_string()))?;

            // Parse tool calls
            let tool_calls = if response.tool_calls.is_empty() {
                vec![ToolCall { tool_name: "no_tool".to_string(), args: serde_json::json!({}), id: None }]
            } else {
                response.tool_calls.iter().map(|tc| ToolCall {
                    tool_name: tc.function.name.clone(),
                    args: serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::json!({})),
                    id: Some(tc.id.clone()),
                }).collect()
            };

            // Execute tools
            let mut tool_results = Vec::new();
            let mut next_prompts = Vec::new();
            let mut exit_reason = None;

            for tc in tool_calls {
                let handler = tools.iter()
                    .find(|t| t.name() == tc.tool_name)
                    .ok_or_else(|| AgentError::ToolError(format!("unknown tool: {}", tc.tool_name)))?;

                let outcome = handler.execute(tc.args, &mut context).await
                    .map_err(|e| AgentError::ToolError(e.to_string()))?;

                if outcome.should_exit {
                    exit_reason = Some(ExitReason::Exited { data: outcome.data });
                    break;
                }

                if outcome.next_prompt.is_none() {
                    exit_reason = Some(ExitReason::CurrentTaskDone { data: outcome.data });
                    break;
                }

                if let Some(prompt) = outcome.next_prompt {
                    next_prompts.push(prompt);
                }

                if let Some(data) = &outcome.data {
                    if let Some(id) = &tc.id {
                        let content = match data {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        tool_results.push(ToolResult {
                            tool_use_id: id.clone(),
                            content,
                        });
                    }
                }
            }

            if let Some(reason) = exit_reason {
                return Ok(RunResult { reason, turns: turn });
            }

            // Build next message
            let next_prompt = next_prompts.join("\n");
            messages.push(Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: next_prompt }],
                tool_results: Some(tool_results),
            });
        }

        Ok(RunResult {
            reason: ExitReason::MaxTurnsExceeded,
            turns: turn,
        })
    }
}

#[derive(Debug)]
pub struct RunResult {
    pub reason: ExitReason,
    pub turns: u32,
}

#[derive(Debug, Clone)]
pub enum ExitReason {
    Exited { data: Option<serde_json::Value> },
    CurrentTaskDone { data: Option<serde_json::Value> },
    MaxTurnsExceeded,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    LlmError(String),
    #[error("Tool error: {0}")]
    ToolError(String),
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add ga-core/src/agent/loop.rs
git commit -m "feat(rust): implement Agent main loop"
```

---

### Task 8: 实现 HTTP API 路由

**目标**: 实现 REST API 端点，暴露 Agent 功能

**文件:**
- Create: `ga-core/src/api/routes.rs`
- Modify: `ga-core/src/api/mod.rs`
- Modify: `ga-core/src/main.rs`

- [ ] **Step 1: 实现 api/routes.rs**

```rust
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::agent::loop::{AgentLoop, ExitReason, RunResult};
use crate::llm::client::ToolSchema;
use crate::tools::{ToolContext, ToolHandler, WorkingMemory};

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub system_prompt: String,
    pub user_input: String,
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
    #[serde(default = "default_verbose")]
    pub verbose: bool,
}

fn default_max_turns() -> u32 { 40 }
fn default_verbose() -> bool { true }

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub result: String,
    pub data: Option<serde_json::Value>,
    pub turns: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Arc<dyn ToolHandler>>,
    pub tools_schema: Vec<ToolSchema>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/agent/run", post(run_agent))
        .route("/api/schema", get(get_schema))
        .with_state(state)
}

async fn run_agent(
    State(state): State<AppState>,
    Json(req): Json<RunRequest>,
) -> Json<RunResponse> {
    // TODO: 需要接入实际的 LLM client
    // 这里先用 mock 实现
    Json(RunResponse {
        result: "CURRENT_TASK_DONE".to_string(),
        data: None,
        turns: 1,
    })
}

async fn get_schema(
    State(state): State<AppState>,
) -> Json<Vec<ToolSchema>> {
    Json(state.tools_schema.clone())
}
```

- [ ] **Step 2: 更新 api/mod.rs**

```rust
pub mod routes;
pub mod websocket;

pub use routes::{AppState, RunRequest, RunResponse, create_router};
```

- [ ] **Step 3: 更新 main.rs**

```rust
use ga_core::api::{AppState, create_router};
use ga_core::tools::{FileReadTool, FilePatchTool, FileWriteTool, CodeRunTool};
use ga_core::llm::client::ToolSchema;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let tools: Vec<Arc<dyn ga_core::tools::ToolHandler>> = vec![
        Arc::new(FileReadTool),
        Arc::new(FilePatchTool),
        Arc::new(FileWriteTool),
        Arc::new(CodeRunTool),
    ];

    // TODO: 从 tools_schema.json 加载
    let tools_schema = vec![];

    let state = AppState { tools, tools_schema };
    let app = create_router(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("ga-core listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 4: 编译验证**

```bash
cargo check
```

- [ ] **Step 5: Commit**

```bash
git add ga-core/src/api/routes.rs ga-core/src/api/mod.rs ga-core/src/main.rs
git commit -m "feat(rust): add HTTP API routes for agent run and schema"
```

---

### Task 9: 实现 Claude LLM 客户端

**目标**: 实现 Claude API 客户端

**文件:**
- Create: `ga-core/src/llm/claude.rs`

- [ ] **Step 1: 实现 claude.rs**

```rust
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

use super::client::{LlmClient, LlmError, ToolSchema};
use super::models::{ContentBlock, Message, MockFunction, MockResponse, MockToolCall, Role};

pub struct ClaudeSession {
    api_key: String,
    api_base: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeSession {
    pub fn new(api_key: String, api_base: String, model: String) -> Self {
        Self { api_key, api_base, model, client: reqwest::Client::new() }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers
    }
}

#[async_trait]
impl LlmClient for ClaudeSession {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError> {
        let url = format!("{}/v1/messages", self.api_base);

        let mut body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        let resp = self.client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await
                .map_err(|e| LlmError::ParseError(e.to_string()))?;

            // Parse response
            let content = json["content"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v["text"].as_str())
                    .collect::<Vec<_>>()
                    .join(""))
                .unwrap_or_default();

            let tool_calls: Vec<MockToolCall> = json["content"].as_array()
                .map(|arr| arr.iter()
                    .filter(|v| v["type"] == "tool_use")
                    .map(|v| MockToolCall {
                        function: MockFunction {
                            name: v["name"].as_str().unwrap_or("").to_string(),
                            arguments: v["input"].to_string(),
                        },
                        id: v["id"].as_str().unwrap_or("").to_string(),
                    })
                    .collect())
                .unwrap_or_default();

            Ok(MockResponse {
                thinking: String::new(),
                content,
                tool_calls,
                raw: json.to_string(),
                stop_reason: json["stop_reason"].as_str().unwrap_or("").to_string(),
            })
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(LlmError::ApiError(text))
        }
    }

    async fn chat_stream(
        &self,
        _messages: Vec<Message>,
        _tools: Option<Vec<ToolSchema>>,
    ) -> Result<futures::stream::BoxStream<'static, Result<String, LlmError>>, LlmError> {
        // TODO: 实现 SSE 流式输出
        todo!("stream not yet implemented")
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add ga-core/src/llm/claude.rs
git commit -m "feat(rust): implement Claude LLM client"
```

---

### Task 10: 集成测试与验证

**目标**: 验证整个系统可以编译运行，API 可访问

**文件:**
- Create: `ga-core/tests/integration_tests.rs`

- [ ] **Step 1: 实现集成测试**

```rust
use reqwest;

#[tokio::test]
async fn test_health_endpoint() {
    // 启动服务器（需要修改 main.rs 支持可测试性）
    // 简化测试：假设服务器已在运行
    let resp = reqwest::get("http://localhost:8080/health").await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "OK");
}
```

- [ ] **Step 2: 运行测试**

```bash
cargo test
```

Expected: 编译通过，单元测试通过

- [ ] **Step 3: 手动验证**

```bash
cargo run &
sleep 2
curl http://localhost:8080/health
curl http://localhost:8080/api/schema
```

Expected: health 返回 OK，schema 返回 JSON 数组

- [ ] **Step 4: Commit**

```bash
git add ga-core/tests/integration_tests.rs
git commit -m "test(rust): add integration tests"
```

---

## 后续任务（Phase 2）

以下任务在 Phase 1 骨架完成后继续：

- [ ] **Task 11**: 实现剩余工具（web.rs, memory.rs, user.rs）
- [ ] **Task 12**: 实现 OpenAI 客户端（llm/openai.rs）
- [ ] **Task 13**: 实现 MixinSession（llm/mixin.rs）
- [ ] **Task 14**: 实现浏览器桥接（browser/driver.rs, browser/session.rs）
- [ ] **Task 15**: 实现 HTML 简化（utils/html.rs）
- [ ] **Task 16**: 实现流式输出 WebSocket（api/websocket.rs）
- [ ] **Task 17**: 加载 tools_schema.json 并注册所有工具
- [ ] **Task 18**: 实现 AgentHandler 状态管理
- [ ] **Task 19**: 实现全局记忆读写
- [ ] **Task 20**: Python 前端客户端封装

---

## 验证清单

Phase 1 完成时，应满足：

- [ ] `cargo test` 全部通过
- [ ] `cargo run` 启动 HTTP 服务
- [ ] `curl http://localhost:8080/health` 返回 `OK`
- [ ] `curl http://localhost:8080/api/schema` 返回工具 schema JSON
- [ ] 文件操作工具单元测试通过
- [ ] Agent Loop 可编译通过（尚未接入实际 LLM）
