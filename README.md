# cx —— 终端优先的自主 AI Agent 执行系统

`cx` 是一个用 Rust 编写的**终端优先、可自我进化**的 AI Agent。它只给你一个命令行界面（CLI）和 TUI（文本用户界面），不依赖浏览器或 Web 服务，专为开发者、运维人员、自动化爱好者在**纯终端环境**下使用而设计。

> 项目处于 **MVP 开发阶段**，遵循分阶段交付。当前代码库已完成模块骨架和部分工具抽象，核心执行循环、TUI、技能固化等功能正在逐步实现中。

## ✨ 设计目标

- **终端优先**：所有交互通过 TUI 或单次 CLI 命令完成，完全支持 SSH 远程使用。
- **极低资源占用**：空闲内存 < 50 MB，启动时间 < 100 ms。
- **自我进化**：成功执行的任务路径会被自动固化为可复用的技能（Skill），下次遇到类似任务时直接调用，无需重复消耗 LLM tokens。
- **单二进制部署**：无 Python、Node.js 等运行时依赖，一个文件即可运行。
- **跨平台**：支持 Windows、macOS、Linux。

## 🧠 架构概览

`cx` 遵循 GenericAgent 的极简四层记忆系统（L0 人格，L1 会话，L2 用户，L3 技能）和 9 个原子工具。其核心循环仅约 150 行代码，通过 LLM 驱动工具调用完成任务。

> 📘 详细设计请参见 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## 🚀 快速开始（当前可用部分）

### 安装

```bash
# 克隆仓库
git clone https://github.com/CHQ7/cx.git
cd cx

# 构建 release 版本
cargo build --release

# 二进制文件位于 target/release/cx