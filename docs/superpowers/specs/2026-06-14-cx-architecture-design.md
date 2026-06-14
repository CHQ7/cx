# CX 架构设计文档

> **日期**: 2026-06-14  
> **状态**: 设计中  
> **目标**: 将 CX 从骨架项目转化为独立可用的 Agent 系统

---

## 1. 项目定位

### 1.1 什么是什么

CX 是一个独立的、终端优先的 AI Agent 执行系统，核心特性：

- **Rust 实现**: 单二进制部署，无 Python 依赖
- **双模式运行**: 本地 CLI 执行 或 HTTP 服务模式
- **多 LLM 支持**: Claude、OpenAI、Mixin 故障转移
- **Docker 原生**: 支持容器化部署

### 1.2 与 GA2 的关系

| 维度 | GA2 | CX |
|------|-----|-----|
| 语言 | Python | Rust |
| 架构 | 进程内 SDK | 本地 CLI + HTTP 服务 |
| 浏览器 | TMWebDriver (Python) | **MVP 暂不支持** |
| 配置 | mykey.py | config.toml + 热重载 |
| 部署 | 本地运行 | 本地 + Docker |

**原则**: CX 完全独立设计，不追求与 GA2 的 API 兼容。

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                         CX                                  │
│                    (单一二进制 cx)                           │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   CLI 层     │  │   核心引擎    │  │   HTTP 服务层    │  │
│  │              │  │              │  │                  │  │
│  │ cx run       │  │  Agent Loop  │  │  axum server     │  │
│  │ cx server    │  │  Tool 调度   │  │  /api/run        │  │
│  │ cx config    │  │  LLM 客户端   │  │  /api/stream     │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│         └─────────────────┴────────────────────┘            │
│                           │                                 │
│  ┌────────────────────────▼────────────────────────────┐   │
│  │                   工具层                             │   │
│  │  file_read │ file_patch │ file_write │ code_run    │   │
│  │  ask_user  │ update_working_checkpoint              │   │
│  │  (web_scan │ web_execute_js - MVP 暂不实现)         │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                 │
│  ┌────────────────────────▼────────────────────────────┐   │
│  │                配置管理层                            │   │
│  │   多 provider │ 热重载 │ Mixin 故障转移              │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 运行模式

#### 模式 A: 本地 CLI 模式（默认）

```bash
$ cx run "帮我写一个 Rust HTTP 服务"

✦ CX · model: claude-opus
Turn 1 ...
🛠️ Tool: code_run
   args: { script: "cargo new http-server", ... }
   ...
Done!
```

**特点**:
- 内嵌 Agent Loop，直接执行
- 流式输出到终端
- 适合本地开发、脚本集成

#### 模式 B: HTTP 服务模式

```bash
$ cx server --port 8080

[INFO] CX server listening on 0.0.0.0:8080
[INFO] Loaded 3 LLM configurations
```

**特点**:
- axum HTTP 服务
- 支持 SSE 流式输出
- 适合 Docker 部署、远程调用

#### 模式 C: CLI 客户端模式

```bash
$ cx run "任务" --remote http://cx-server:8080
```

**特点**:
- CLI 作为客户端连接远程服务
- 保持相同的用户体验
- 适合分布式部署

---

## 3. 模块划分

```
cx/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口，命令解析
│   ├── lib.rs               # 库入口
│   ├── cli/                 # CLI 层
│   │   ├── mod.rs
│   │   ├── commands.rs      # 子命令定义
│   │   ├── run.rs           # cx run 实现
│   │   ├── server.rs        # cx server 实现
│   │   └── config.rs        # cx config 实现
│   ├── core/                # 核心引擎
│   │   ├── mod.rs
│   │   ├── agent.rs         # Agent Loop
│   │   ├── session.rs       # 会话管理
│   │   └── dispatcher.rs    # 工具调度
│   ├── llm/                 # LLM 客户端
│   │   ├── mod.rs
│   │   ├── client.rs        # LlmClient trait
│   │   ├── claude.rs        # Claude 实现
│   │   ├── openai.rs        # OpenAI 实现
│   │   ├── mixin.rs         # Mixin 故障转移
│   │   ├── models.rs        # 数据类型
│   │   └── stream.rs        # 流式输出处理
│   ├── tools/               # 工具实现
│   │   ├── mod.rs
│   │   ├── file_ops.rs      # 文件操作
│   │   ├── code_run.rs      # 代码执行
│   │   ├── memory.rs        # 工作记忆
│   │   └── user.rs          # 用户交互
│   ├── config/              # 配置管理
│   │   ├── mod.rs
│   │   ├── manager.rs       # 配置管理器（热重载）
│   │   ├── provider.rs      # Provider 配置
│   │   └── store.rs         # 配置存储
│   ├── server/              # HTTP 服务
│   │   ├── mod.rs
│   │   ├── app.rs           # axum app 构建
│   │   ├── handlers.rs      # 请求处理
│   │   └── stream.rs        # SSE 流式
│   └── utils/               # 工具函数
│       ├── mod.rs
│       ├── paths.rs         # 路径管理
│       └── output.rs        # 输出格式化
├── tests/                   # 集成测试
└── examples/                # 示例配置
```

---

## 4. 核心数据类型

### 4.1 LLM 消息系统

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
}
```

### 4.2 工具系统

```rust
// src/tools/mod.rs

#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> serde_json::Value;

    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &mut ToolContext,
    ) -> Result<ToolOutput, ToolError>;
}

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub current_turn: u32,
    pub working_dir: PathBuf,
    pub working_memory: WorkingMemory,
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub result: serde_json::Value,
    pub next_prompt: Option<String>,
    pub should_exit: bool,
}
```

### 4.3 流式输出

```rust
// src/core/stream.rs

pub enum StreamEvent {
    /// 开始新回合
    TurnStart { turn: u32 },
    /// LLM 思考内容（流式）
    Thinking { text: String },
    /// LLM 回复内容（流式）
    Content { text: String },
    /// 工具调用开始
    ToolStart { name: String, args: serde_json::Value },
    /// 工具执行结果
    ToolResult { name: String, result: serde_json::Value },
    /// 回合结束
    TurnEnd { turn: u32 },
    /// 任务完成
    Done { reason: ExitReason },
    /// 错误
    Error { message: String },
}
```

---

## 5. CLI 设计

### 5.1 命令结构

```bash
cx [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

Global Options:
  -c, --config <PATH>     配置文件路径 (默认: ~/.cx/config.toml)
  -v, --verbose           详细输出
  -q, --quiet             静默模式
  -h, --help              帮助信息
  -V, --version           版本信息
```

### 5.2 子命令

#### cx run - 执行任务

```bash
cx run [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>  任务描述（支持从 stdin 读取）

Options:
  -m, --model <NAME>      指定 LLM 模型
  -n, --max-turns <N>     最大回合数 (默认: 40)
  --remote <URL>          连接远程服务
  --no-stream             非流式输出
  -o, --output <FILE>     输出到文件

Examples:
  cx run "帮我写个快速排序"
  echo "优化这段代码" | cx run --model gpt-4
  cx run "分析日志" < app.log
```

#### cx server - 启动服务

```bash
cx server [OPTIONS]

Options:
  -p, --port <PORT>       监听端口 (默认: 8080)
  -H, --host <HOST>       监听地址 (默认: 127.0.0.1)
  --no-hot-reload         禁用配置热重载

Examples:
  cx server
  cx server --port 8080 --host 0.0.0.0
```

#### cx config - 配置管理

```bash
cx config <SUBCOMMAND>

Subcommands:
  list                    列出所有配置
  show <NAME>             显示配置详情
  add <NAME>              添加新配置（交互式）
  edit <NAME>             编辑配置
  remove <NAME>           删除配置
  switch <NAME>           切换默认配置
  test <NAME>             测试配置连接
  reload                  热重载配置

Examples:
  cx config list
  cx config add claude-prod
  cx config switch claude-prod
  cx config test openai-backup
```

---

## 6. 配置管理

### 6.1 配置文件格式

```toml
# ~/.cx/config.toml

# 默认使用的配置
default = "claude-prod"

# 全局设置
[global]
max_turns = 40
verbose = true
stream = true

# Claude 生产环境
[[providers]]
name = "claude-prod"
type = "claude"
api_key = "${CLAUDE_API_KEY}"  # 支持环境变量
api_base = "https://api.anthropic.com"
model = "claude-opus-4"
max_tokens = 4096

# Claude 备用
[[providers]]
name = "claude-backup"
type = "claude"
api_key = "${CLAUDE_BACKUP_KEY}"
api_base = "https://api.anthropic.com"
model = "claude-sonnet-4"

# OpenAI
[[providers]]
name = "gpt-4"
type = "openai"
api_key = "${OPENAI_API_KEY}"
api_base = "https://api.openai.com"
model = "gpt-4o"

# Mixin 故障转移配置
[[providers]]
name = "mixin-primary"
type = "mixin"
strategy = "fallback"  # fallback | round_robin
providers = ["claude-prod", "claude-backup", "gpt-4"]
```

### 6.2 配置热重载

```rust
// src/config/manager.rs

pub struct ConfigManager {
    path: PathBuf,
    current: Arc<RwLock<Config>>,
    watcher: RecommendedWatcher,
}

impl ConfigManager {
    /// 加载配置
    pub fn load(path: &Path) -> Result<Self, ConfigError>;

    /// 获取当前配置（线程安全）
    pub fn get(&self) -> Arc<Config>;

    /// 手动重载
    pub fn reload(&self) -> Result<(), ConfigError>;

    /// 切换默认 provider
    pub fn switch_default(&self, name: &str) -> Result<(), ConfigError>;
}
```

---

## 7. API 设计（HTTP 模式）

### 7.1 REST Endpoints

```
POST   /api/run              # 执行任务（流式 SSE）
POST   /api/run/sync         # 执行任务（同步 JSON）
GET    /api/health           # 健康检查
GET    /api/config           # 获取配置列表
POST   /api/config/reload    # 热重载配置
GET    /api/providers        # 获取 provider 列表
```

### 7.2 请求/响应格式

#### POST /api/run (SSE 流式)

**Request:**
```json
{
  "prompt": "帮我写个快速排序",
  "model": "claude-opus",
  "max_turns": 40,
  "stream": true
}
```

**Response (SSE):**
```
event: turn_start
data: {"turn": 1}

event: content
data: {"text": "我来帮你"}

event: tool_start
data: {"name": "code_run", "args": {"script": "..."}}

event: tool_result
data: {"name": "code_run", "result": {"status": "success"}}

event: done
data: {"reason": "task_complete", "turns": 3}
```

---

## 8. Docker 部署

### 8.1 Dockerfile

```dockerfile
FROM rust:1.80-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/target/release/cx /usr/local/bin/cx
EXPOSE 8080
ENTRYPOINT ["cx"]
CMD ["server", "--host", "0.0.0.0"]
```

### 8.2 Docker Compose

```yaml
version: '3.8'

services:
  cx:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./config:/root/.cx
      - ./workspace:/workspace
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
    working_dir: /workspace
```

### 8.3 使用方式

```bash
# 启动服务
docker-compose up -d

# CLI 客户端连接
cx run "任务" --remote http://localhost:8080

# 或者进入容器执行
docker-compose exec cx cx run "任务"
```

---

## 9. 开发路线图

### Phase 1: MVP（核心可用）

- [ ] 配置管理系统（热重载、多 provider）
- [ ] Agent Loop 接入 CLI（cx run 本地模式）
- [ ] 流式输出（终端实时显示）
- [ ] 基础工具（file_ops, code_run, ask_user）
- [ ] HTTP 服务模式（cx server）
- [ ] CLI 客户端模式（--remote）

### Phase 2: 完整功能

- [ ] Mixin 故障转移
- [ ] 完整工具集（memory, long_term_update）
- [ ] 会话持久化
- [ ] WebSocket 支持
- [ ] 性能优化

### Phase 3: 高级功能

- [ ] 浏览器支持（Playwright/CDP 集成）
- [ ] 插件系统
- [ ] 分布式部署
- [ ] Web UI

---

## 10. 验收标准

### MVP 完成标准

- [ ] `cx run "任务"` 可完成一个完整的代码编写任务
- [ ] 支持至少 2 个 LLM provider（Claude + OpenAI）
- [ ] 配置热重载工作正常
- [ ] `cx server` + `cx run --remote` 可正常工作
- [ ] Docker 镜像可正常运行
- [ ] 单元测试覆盖率 > 60%

---

## 附录

### A. 命名规范

| 层级 | 命名 | 示例 |
|------|------|------|
| 二进制 | `cx` | `cx run` |
| 库 crate | `cx-core` | `use cx_core::Agent;` |
| 配置目录 | `~/.cx/` | `~/.cx/config.toml` |
| 工作目录 | `./.cx/` | `./.cx/memory/` |

### B. 环境变量

| 变量 | 说明 |
|------|------|
| `CX_CONFIG` | 配置文件路径 |
| `CX_HOME` | CX 数据目录 |
| `CX_LOG` | 日志级别 |
