# Tools 功能实现 - 任务规划

## 项目背景

在 nanobot-agent crate 中实现工具（Tools）功能，使 AI Agent 能够执行文件系统操作和 Shell 命令。

## 技术约束

- **语言**: Rust (Edition 2024)
- **异步运行时**: Tokio
- **序列化**: serde + serde_json
- **Schema 定义**: schemars 或自定义 JSON Schema 生成
- **错误处理**: thiserror
- **日志**: tracing

## 目录结构规划

新的 `nanobot-tools` crate 将独立于 agent crate：

```
crates/
├── agent/                  # agent crate (依赖 nanobot-tools)
│   ├── Cargo.toml          # 添加 nanobot-tools 依赖
│   └── src/
│       ├── lib.rs          # 删除 tools 模块导入，添加 nanobot-tools 依赖
│       ├── schema/mod.rs   # 已有：消息定义
│       └── loop.rs         # 修改：集成 ToolRegistry
│
└── tools/                  # 新的 nanobot-tools crate
    ├── Cargo.toml          # 新的 crate 配置
    └── src/
        ├── lib.rs          # 模块入口
        ├── core.rs         # 基础工具抽象（Tool trait, ToolResult, ToolError）
        ├── registry.rs     # 工具注册表（ToolRegistry）
        ├── fs.rs           # 文件系统工具（read_file, write_file, edit_file, list_dir）
        ├── shell.rs        # Shell 执行工具
        └── tests.rs        # 单元测试
```

### 依赖关系
- `nanobot-tools`: 独立 crate，依赖 schemars, serde, tokio, thiserror, tracing 等
- `nanobot-agent`: 依赖 `nanobot-tools`，通过 `ToolRegistry` 集成工具功能

---

## 任务清单

### 任务 1: 添加依赖并创建 tools 模块目录结构
**优先级**: P0 | **预估工时**: 30分钟

**验收标准**:
- [ ] 在 `crates/agent/Cargo.toml` 中添加 `schemars` 和 `tokio-util` 依赖
- [ ] 创建 `crates/agent/src/tools/` 目录
- [ ] 创建基础文件：`mod.rs`, `core.rs`, `registry.rs`, `fs.rs`, `shell.rs` (空文件或最小化结构)
- [ ] 更新 `crates/agent/src/lib.rs`，导出 `tools` 模块

---

### 任务 2: 实现基础工具抽象层
**优先级**: P0 | **预估工时**: 1小时

**验收标准**:
- [ ] 在 `core.rs` 中定义 `Tool` trait，包含以下方法：`name()`, `description()`, `parameters()`, `execute()`, `to_schema()`
- [ ] 定义 `ToolResult` 类型，支持字符串输出和结构化输出
- [ ] 使用 `thiserror` 定义 `ToolError`，包含：`Validation`, `Execution`, `NotFound`, `PermissionDenied`, `Timeout`
- [ ] 实现参数验证逻辑，使用 JSON Schema 验证输入参数

---

### 任务 3: 实现工具注册表
**优先级**: P0 | **预估工时**: 1小时

**验收标准**:
- [ ] 在 `registry.rs` 中实现 `ToolRegistry` 结构体，使用 `HashMap<String, Box<dyn Tool>>` 存储工具
- [ ] 实现 `register()` 方法，注册工具实例
- [ ] 实现 `unregister()` 方法，按名称注销工具
- [ ] 实现 `get()` 方法，获取指定名称的工具
- [ ] 实现 `get_definitions()` 方法，返回所有工具的 OpenAI Function Calling 格式定义
- [ ] 实现 `tool_names()` 方法，返回所有已注册工具名称列表
- [ ] 实现 `execute(name, params).await` 异步执行工具
- [ ] 当工具不存在时返回清晰的错误信息，列出所有可用工具

---

### 任务 4: 实现 read_file 文件系统工具
**优先级**: P0 | **预估工时**: 1小时

**验收标准**:
- [ ] 在 `fs.rs` 中实现 `ReadFileTool` 结构体
- [ ] 定义 JSON Schema: `{ path: string }`
- [ ] 支持相对路径（基于 workspace）和绝对路径
- [ ] 将相对路径解析为相对于 workspace 的绝对路径
- [ ] 检查文件是否存在且为普通文件，否则返回明确错误
- [ ] 检查路径是否在允许目录内（使用 `allowed_dir` 限制），超出则返回权限错误
- [ ] 读取文件内容并返回字符串

---

### 任务 5: 实现 write_file 文件系统工具
**优先级**: P0 | **预估工时**: 1小时

**验收标准**:
- [ ] 在 `fs.rs` 中实现 `WriteFileTool` 结构体
- [ ] 定义 JSON Schema: `{ path: string, content: string }`
- [ ] 自动创建父目录（使用 `tokio::fs::create_dir_all`）
- [ ] 支持相对路径和绝对路径处理
- [ ] 检查路径权限限制
- [ ] 写入文件并返回成功确认信息（包含写入字节数或文件路径）

---

### 任务 6: 实现 edit_file 文件系统工具
**优先级**: P1 | **预估工时**: 2小时

**验收标准**:
- [ ] 在 `fs.rs` 中实现 `EditFileTool` 结构体
- [ ] 定义 JSON Schema: `{ path: string, old_text: string, new_text: string }`
- [ ] 要求 `old_text` 完全匹配（至少包含 3 行上下文确保唯一性）
- [ ] 当 `old_text` 不匹配时，返回详细的差异提示
- [ ] 当 `old_text` 出现多次时，警告用户并要求提供更唯一的上下文
- [ ] 支持相对路径和绝对路径，检查权限限制
- [ ] 安全地替换文件内容，返回成功确认

---

### 任务 7: 实现 list_dir 文件系统工具
**优先级**: P1 | **预估工时**: 1小时

**验收标准**:
- [ ] 在 `fs.rs` 中实现 `ListDirTool` 结构体
- [ ] 定义 JSON Schema: `{ path: string, recursive?: boolean }`
- [ ] 列出指定目录的内容（文件和子目录），返回格式化列表
- [ ] 支持 `recursive` 选项进行递归列出（可选）
- [ ] 支持相对路径和绝对路径，检查权限限制
- [ ] 当路径不存在或不是目录时，返回明确错误

---

### 任务 8: 实现 Shell 命令执行工具
**优先级**: P1 | **预估工时**: 2小时

**验收标准**:
- [ ] 在 `shell.rs` 中实现 `ShellTool` 结构体
- [ ] 定义 JSON Schema: `{ command: string, cwd?: string, timeout_ms?: number }`
- [ ] 支持设置工作目录（可选，默认为 workspace）
- [ ] 支持设置环境变量（可选）
- [ ] 实现基本的安全检查：阻止包含危险关键字（`rm -rf /`, `mkfs`, `dd if=/dev/zero`, `format` 等）的命令
- [ ] 支持超时控制（默认 30 秒），超时后终止进程
- [ ] 处理 stdin 自动拒绝（命令需要交互时返回错误提示）
- [ ] 返回完整执行结果：`{ exit_code: number, stdout: string, stderr: string }`
- [ ] 输出超出最大长度（如 10000 字符）时进行截断并标记 `(truncated)`

---

### 任务 9: 实现错误处理和日志记录
**优先级**: P1 | **预估工时**: 1小时

**验收标准**:
- [ ] 所有工具错误返回格式化的错误字符串，包含错误类型和描述
- [ ] 使用 `tracing` 记录工具调用日志（info 级别：工具名称、参数摘要）
- [ ] 使用 `tracing` 记录错误日志（error 级别：工具名称、参数、错误详情）
- [ ] 参数验证失败时，错误信息指明具体字段和期望值
- [ ] 工具执行超时时，错误信息包含超时时间和实际执行时长
- [ ] 权限错误发生时，错误信息说明受限制的路径和允许的操作范围

---

### 任务 10: 集成到 AgentLoop 并初始化
**优先级**: P1 | **预估工时**: 1.5小时

**验收标准**:
- [ ] 修改 `crates/agent/src/loop.rs`，在 `AgentLoop` 中添加 `ToolRegistry` 字段
- [ ] 创建 `AgentLoop::with_tools()` 构造函数，接收配置并初始化工具
- [ ] 从配置中读取 `workspace`、`allowed_dir`、`timeout` 等参数
- [ ] 默认注册文件系统工具（read_file, write_file, edit_file, list_dir）
- [ ] 默认注册 Shell 工具
- [ ] 在 `process_direct` 或消息处理循环中支持工具调用解析和执行
- [ ] 确保工具配置可以从 `AgentDefaults` 中读取

---

### 任务 11: 编写单元测试
**优先级**: P2 | **预估工时**: 2小时

**验收标准**:
- [ ] 测试 `ToolRegistry`：空注册、批量注册、注销、获取定义
- [ ] 测试 `MockTool` 辅助实现（如果 Tool trait 是 object-safe）
- [ ] 测试 `read_file`：正常读取、路径解析、权限限制、文件不存在错误
- [ ] 测试 `write_file`：正常写入、自动创建目录、权限限制
- [ ] 测试 `edit_file`：正常编辑、不匹配时的差异提示、多次匹配警告
- [ ] 测试 `list_dir`：正常列出、递归列出、无效路径错误
- [ ] 测试 `shell`：正常执行、超时处理、危险命令拦截、输出截断
- [ ] 使用临时目录进行文件系统测试，不污染实际文件系统
- [ ] 确保测试覆盖率不低于 80%

---

## 任务依赖关系

```
任务 1 (依赖) ────────────────────────────────────────────────┐
  ↓                                                           │
任务 2 (工具抽象) ────────────────────────────────────────────┤
  ↓                                                           │
任务 3 (注册表) ←─────────────────────────────────────────────┤
  ↓                                                           │
任务 4-8 (具体工具) ──────────────────────────────────────────┤
  ↓                                                           │
任务 9 (错误处理) ←───────────────────────────────────────────┤
  ↓                                                           │
任务 10 (AgentLoop 集成) ←────────────────────────────────────┤
  ↓                                                           │
任务 11 (测试) ←──────────────────────────────────────────────┘

依赖说明：
- 任务 4-8 可以并行开发，都依赖任务 2 和 3
- 任务 9 依赖于任务 2-8
- 任务 10 依赖于任务 1-9
- 任务 11 依赖于任务 1-10
```

---

## 进度追踪

| 任务 | 状态 | 负责人 | 实际工时 | 备注 |
|------|------|--------|----------|------|
| 1    | ✅ 已完成 | AI | 30分钟 | 创建了 nanobot-tools crate |
| 2    | ✅ 已完成 | AI | 30分钟 | core.rs 实现完成 |
| 3    | ✅ 已完成 | AI | 30分钟 | registry.rs 实现完成 |
| 4    | ✅ 已完成 | AI | 30分钟 | ReadFileTool 实现完成 |
| 5    | ✅ 已完成 | AI | 30分钟 | WriteFileTool 实现完成 |
| 6    | ✅ 已完成 | AI | 30分钟 | EditFileTool 实现完成 |
| 7    | ✅ 已完成 | AI | 20分钟 | ListDirTool 实现完成 |
| 8    | ✅ 已完成 | AI | 30分钟 | ShellTool 实现完成 |
| 9    | ✅ 已完成 | AI | 20分钟 | 错误处理和日志已集成 |
| 10   | ⏳ 待开始 | - | - | - |
| 11   | ⏳ 待开始 | - | - | - |

---

## 变更历史

| 日期 | 版本 | 变更说明 | 作者 |
|------|------|----------|------|
| 2026-03-01 | 1.0 | 初始版本，包含 7 个需求、11 个任务 | AI |
