# GenericAgent Rust Core 设计文档

> **日期**: 2026-06-14
> **范围**: 仅核心（Agent Loop + 9 个原子工具 + LLM Core）
> **集成方式**: HTTP API 服务（axum）
> **数据持久化**: 文件系统（兼容现有 Python 格式）

---

## 1. 背景与目标

### 1.1 现状

GenericAgent 当前为纯 Python 实现，核心代码约 1.8K 行：

| 模块 | 行数 | 职责 |
|------|------|------|
| `agent_loop.py` | 133 | Agent 主循环，工具分发 |
| `ga.py` | 592 | 9 个原子工具的实现 |
| `llmcore.py` | 1073 | LLM 客户端抽象（Claude/OpenAI/Mixin） |
| `TMWebDriver.py` | 282 | 浏览器桥接（WebSocket/HTTP） |

### 1.2 目标

将核心逻辑迁移至 Rust，获得：
- **性能**: 更低的内存占用和更高的执行效率
- **可靠性**: 编译期类型检查消除运行时错误
- **部署**: 单二进制文件，无 Python 依赖

Python 前端（TUI、Streamlit、IM Bot）通过 HTTP API 调用 Rust 核心。

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Python Frontends                        │
│  (TUI v3, Streamlit, Qt, Telegram, WeChat, DingTalk...)   │
└─────────────────────────┬───────────────────────────────────┘
                          │ HTTP / WebSocket
┌─────────────────────────▼───────────────────────────────────┐
│                      ga-core (Rust)                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   API Layer │  │  Agent Loop │  │    LLM Clients      │ │
│  │   (axum)    │  │  (handler)  │  │  (Claude/OpenAI)    │ │
│  └─────────────┘  └──────┬──────┘  └─────────────────────┘ │
│                          │                                   │
│  ┌───────────────────────▼───────────────────────────────┐ │
│  │              9 Atomic Tools                          │ │
│  │  code_run │ file_read │ file_patch │ file_write      │ │
│  │  web_scan │ web_execute_js │ update_working_checkpoint │ │
│  │  ask_user │ start_long_term_update                     │ │
│  └───────────────────────────────────────────────────────┘ │
│                          │                                   │
│  ┌───────────────────────▼───────────────────────────────┐ │
│  │              Browser Bridge (TMWebDriver)            │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 模块划分

```
ga-core/
├── Cargo.toml
├── src/
│   ├── main.rs              # HTTP 服务入口
│   ├── lib.rs               # 库入口（可嵌入测试）
│   ├── api/
│   │   ├── mod.rs           # API 路由注册
│   │   ├── routes.rs        # REST endpoints
│   │   └── websocket.rs     # 流式输出 WebSocket
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── loop.rs          # Agent 主循环
│   │   ├── handler.rs       # GenericAgentHandler
│   │   └── outcome.rs       # StepOutcome
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── client.rs        # LlmClient trait
│   │   ├── claude.rs        # ClaudeSession
│   │   ├── openai.rs        # OpenAiSession
│   │   ├── native.rs        # NativeToolClient
│   │   ├── mixin.rs         # MixinSession
│   │   └── models.rs        # Message, ContentBlock, MockResponse
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── code_run.rs      # 代码执行
│   │   ├── file_ops.rs      # 文件读写/补丁
│   │   ├── web.rs           # 浏览器操作
│   │   ├── memory.rs        # 工作记忆/长期记忆
│   │   └── user.rs          # 用户交互
│   ├── browser/
│   │   ├── mod.rs
│   │   ├── driver.rs        # TMWebDriver
│   │   └── session.rs       # Session 管理
│   └── utils/
│       ├── mod.rs
│       ├── html.rs          # HTML 简化
│       └── format.rs        # 字符串格式化
├── tests/
│   └── integration_tests.rs
└── assets/
    └── tools_schema.json    # 工具定义（从 Python 项目复制）
```

---

## 3. 核心数据类型

### 3.1 消息系统（Claude Content-Block 格式）

```rust
// src/llm/models.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

### 3.2 工具调用

```rust
// src/agent/outcome.rs

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: serde_json::Value,
    pub id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub data: Option<serde_json::Value>,
    pub next_prompt: Option<String>,
    pub should_exit: bool,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}
```

### 3.3 LLM 响应

```rust
// src/llm/models.rs

#[derive(Debug, Clone)]
pub struct MockFunction {
    pub name: String,
    pub arguments: String,  // JSON string
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

### 3.4 Agent 状态

```rust
// src/agent/mod.rs

#[derive(Debug, Clone)]
pub struct WorkingMemory {
    pub key_info: Option<String>,
    pub related_sop: Option<String>,
    pub in_plan_mode: Option<String>,
    pub passed_sessions: u32,
}

#[derive(Debug)]
pub struct AgentState {
    pub messages: Vec<Message>,
    pub turn: u32,
    pub max_turns: u32,
    pub working: WorkingMemory,
    pub history_info: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ExitReason {
    Exited { data: serde_json::Value },
    CurrentTaskDone { data: serde_json::Value },
    MaxTurnsExceeded,
}
```

---

## 4. 关键 trait 设计

### 4.1 LlmClient

```rust
// src/llm/client.rs

#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 发送消息并获取完整响应
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError>;

    /// 流式聊天（返回 SSE 流）
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<BoxStream<'static, Result<String, LlmError>>, LlmError>;
}

#[derive(Debug)]
pub enum LlmError {
    ApiError(String),
    ParseError(String),
    NetworkError(String),
    RateLimited,
}
```

### 4.2 ToolHandler

```rust
// src/tools/mod.rs

#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// 工具名称（对应 schema 中的 function.name）
    fn name(&self) -> &'static str;

    /// 执行工具
    async fn execute(
        &self,
        args: serde_json::Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError>;
}

pub struct ToolContext {
    pub current_turn: u32,
    pub working_dir: PathBuf,
    pub working_memory: WorkingMemory,
    pub verbose: bool,
}
```

---

## 5. API 设计（HTTP 端点）

### 5.1 REST API

```
POST /api/agent/run          # 运行一个任务（同步/异步）
POST /api/agent/run/stream   # 流式运行（SSE）
GET  /api/agent/status/:id   # 查询任务状态
POST /api/agent/stop/:id     # 停止任务
POST /api/tools/:name        # 直接调用单个工具
GET  /api/schema             # 获取工具 schema
```

### 5.2 WebSocket API

```
WS /ws/agent/:session_id     # 实时流式会话
```

### 5.3 请求/响应格式

与 Python 版本保持 JSON 兼容：

```rust
// 运行请求
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub system_prompt: String,
    pub user_input: String,
    pub max_turns: Option<u32>,
    pub verbose: Option<bool>,
}

// 运行响应
#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub result: String,           // "EXITED" | "CURRENT_TASK_DONE" | "MAX_TURNS_EXCEEDED"
    pub data: Option<serde_json::Value>,
    pub turns: u32,
}
```

---

## 6. 文件系统兼容性

### 6.1 持久化路径

Rust 核心使用与 Python 版本相同的目录结构：

```rust
// src/utils/paths.rs

pub struct ProjectPaths {
    pub root: PathBuf,           // 项目根目录
    pub memory: PathBuf,         // memory/
    pub temp: PathBuf,           // temp/
    pub assets: PathBuf,         // assets/
}

impl ProjectPaths {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            memory: root.join("memory"),
            temp: root.join("temp"),
            assets: root.join("assets"),
        }
    }
}
```

### 6.2 全局记忆文件

```rust
// memory/global_mem.txt 的读写
pub async fn read_global_memory(path: &Path) -> io::Result<String> {
    fs::read_to_string(path).await
}
```

---

## 7. 与 Python 前端的集成

### 7.1 Python 客户端封装

```python
# Python 前端中的 Rust 客户端封装

import requests
import json

class RustAgentClient:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url

    def run(self, system_prompt: str, user_input: str, max_turns: int = 40):
        resp = requests.post(f"{self.base_url}/api/agent/run", json={
            "system_prompt": system_prompt,
            "user_input": user_input,
            "max_turns": max_turns,
        })
        return resp.json()

    def run_stream(self, system_prompt: str, user_input: str):
        resp = requests.post(
            f"{self.base_url}/api/agent/run/stream",
            json={"system_prompt": system_prompt, "user_input": user_input},
            stream=True,
        )
        for line in resp.iter_lines():
            if line:
                yield json.loads(line)
```

---

## 8. 依赖选型

| 功能 | Crate | 版本 |
|------|-------|------|
| HTTP 服务 | `axum` | ^0.7 |
| 异步运行时 | `tokio` | ^1.40 |
| JSON 序列化 | `serde` + `serde_json` | ^1.0 |
| HTTP 客户端 | `reqwest` | ^0.12 |
| WebSocket | `tokio-tungstenite` | ^0.24 |
| 正则表达式 | `regex` | ^1.10 |
| HTML 解析 | `scraper` | ^0.19 |
| 错误处理 | `thiserror` | ^1.0 |
| 流式处理 | `futures` | ^0.3 |
| 跨进程 | `tokio::process` | (内置) |

---

## 9. 测试策略

### 9.1 单元测试

每个模块的 `#[cfg(test)]`：

```rust
// src/tools/file_ops.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_read() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let result = file_read(path.to_str().unwrap(), None, None, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }
}
```

### 9.2 集成测试

```rust
// tests/integration_tests.rs

#[tokio::test]
async fn test_agent_loop_e2e() {
    // 启动测试服务器
    // 发送 run 请求
    // 验证响应格式
}
```

---

## 10. 风险与决策记录

| 决策 | 选择 | 理由 |
|------|------|------|
| 动态分发 vs 泛型 | `Box<dyn LlmClient>` | 运行时切换 provider 是常见需求 |
| 文件系统 vs 数据库 | 文件系统 | 兼容现有格式，零迁移成本 |
| axum vs actix-web | axum | 与 tokio 生态更一致，编译更快 |
| 流式输出 | SSE / WebSocket | 保持与 Python 版本相同的用户体验 |

---

## 11. 验收标准

- [ ] `cargo test` 全部通过
- [ ] `cargo run` 启动 HTTP 服务，监听 8080
- [ ] Python 前端可通过 HTTP API 调用 Rust 核心完成一次完整任务
- [ ] 9 个原子工具的行为与 Python 版本一致
- [ ] LLM 对话历史格式与 Python 版本兼容
- [ ] 流式输出正常（SSE 或 WebSocket）
