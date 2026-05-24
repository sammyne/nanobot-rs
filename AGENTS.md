# nanobot-rs 开发指南

nanobot-rs 是一个可部署的 AI Agent 框架，支持通过 CLI 交互或作为后台服务接入钉钉/飞书等消息通道。核心流程：接收消息 → 构建上下文 → 调用 LLM → 执行工具 → 返回响应。

## 架构分层

```
nanobot (CLI 入口)
  └── agent (核心循环)
        ├── context  ← memory + skills    # 组装 LLM 输入
        ├── provider ← config             # 调用 LLM（OpenAI/Anthropic）
        ├── tools + mcp                   # 执行工具调用
        ├── session                       # 持久化对话历史
        ├── channels ← config             # 钉钉/飞书消息收发
        ├── subagent                      # 后台子代理
        └── cron / heartbeat              # 定时任务 / 心跳检查
```

## Rust 版本要求

本项目要求 **Rust >= 1.93**。

## 项目结构

```
crates/
├── nanobot/     # [binary] CLI 入口，提供 onboard/agent/gateway/cron 子命令
│                # 依赖 agent, channels, config, cron, heartbeat, provider, session, subagent, templates, utils
├── agent/       # Agent 核心循环，接收消息、构建上下文、调用 LLM、执行工具并返回响应
│                # 依赖 config, context, mcp, provider, tools, session, memory, channels, cron, subagent
├── provider/    # LLM 提供者抽象层，支持 OpenAI 兼容和 Anthropic Messages API
│                # 依赖 config, tools, utils
├── config/      # 统一的配置加载、验证和管理（~/.nanobot/config.json）
│                # 依赖 utils
├── tools/       # 内置工具：文件系统操作（read/write/edit/list）和 Shell 执行
│                # 依赖 utils, config
├── mcp/         # MCP 客户端，将 MCP 服务器工具桥接为统一 Tool 接口
│                # 依赖 config, tools
├── session/     # 会话持久化，以 JSONL 格式存储对话历史
│                # 依赖 provider, utils
├── memory/      # 两层记忆：长期记忆（MEMORY.md）+ 历史日志（HISTORY.md），LLM 驱动整合
│                # 依赖 provider, tools
├── context/     # LLM 上下文构建器，组装系统提示和消息列表
│                # 依赖 provider, memory, skills
├── channels/    # 消息通道抽象及实现（钉钉、飞书）
│                # 依赖 config
├── skills/      # 技能发现、加载和管理（工作空间目录 + 内置技能）
├── subagent/    # 子代理任务管理器，创建和管理后台轻量级代理实例
│                # 依赖 provider, tools, channels, config
├── cron/        # Cron 定时任务调度、存储和执行
│                # 依赖 tools
├── heartbeat/   # 周期性心跳检查，通过两阶段决策避免不必要的代理唤醒
│                # 依赖 provider, tools, config
├── templates/   # 工作空间初始化模板，编译时嵌入（include_dir!）
└── utils/       # 通用工具函数（字符串处理等）
```

每个 crate 目录下有独立的 `AGENTS.md`，包含该 crate 的关键类型和公共 API。

## 工作空间规范

- 所有成员 crate 放在 `crates/` 下，文件夹名不带项目前缀
- 共用依赖声明在 `[workspace.dependencies]`，成员 crate 用 `.workspace = true` 引用

### 依赖声明格式

- **单一配置**用点号语法，**多个配置**用 TOML 表语法：

```toml
# 工作空间根 Cargo.toml

[workspace.dependencies]
thiserror = "1.0"
nanobot-config.path = "crates/config"

[workspace.dependencies.serde]
version = "1.0"
features = ["derive"]

# 成员 crate 的 Cargo.toml

[dependencies]
thiserror.workspace = true
serde.workspace = true
clap = "4.0"  # crate 特有依赖直接声明
```

- 禁止花括号语法（如 `thiserror = { workspace = true }` 或 `serde = { version = "1.0", features = ["derive"] }`）

## 错误处理

| 代码类型 | 错误处理库 |
|---------|-----------|
| 库（library） | `thiserror` — 在 `src/error.rs` 定义错误枚举，枚举值不带 `Error` 前缀或后缀 |
| 可执行文件（binary） | `anyhow` — 用 `.context()` / `.with_context()` 添加语义化上下文 |

```rust
// 库：src/error.rs
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

// 可执行文件
use anyhow::{Context, Result};
let config = std::fs::read_to_string("config.toml")
    .context("failed to read config")?;
```

## 测试实践

### 单元测试

测试代码与源代码分离到同目录的 `tests.rs` 文件：

```
src/modules/hello/
├── mod.rs     # 源代码，末尾加 `#[cfg(test)] mod tests;`
└── tests.rs   # 测试代码，以 `use super::*;` 开头
```

- 禁止将 `#[cfg(test)] mod tests { ... }` 内联在源文件中，必须拆分到独立的 `tests.rs` 文件
- 测试函数名直接描述功能，不加 `test_` 前缀
- 使用 Arrange-Act-Assert 模式

### 模块文件布局

- 禁止同名的 `foo.rs` 文件和 `foo/` 目录共存。当模块需要子模块（如 `tests.rs`）时，必须使用目录形式 `foo/mod.rs`，不得保留同级的 `foo.rs`

### 集成测试

放在 `tests/` 目录，文件名禁止带 `_test` 后缀。

## 文档标准

- 公共 API 用 `///` 文档注释，包含参数、返回值、错误、示例
- 模块级文档用 `//!`

## 代码质量检查

提交前运行：

```bash
cargo +nightly fmt
cargo clippy -- -D warnings -D clippy::uninlined_format_args
cargo test
cargo doc --no-deps
```

## 版本控制

- `.opencode/plans/` 目录下的需求文档和 TODO 文件需要纳入版本控制，提交代码时一并 commit
- 当某个 crate 的公共 API 发生变更时，同步更新对应的 `crates/*/AGENTS.md`
