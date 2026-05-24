# tools crate

内置工具：文件系统操作（read/write/edit/list）和 Shell 执行。

## 架构

```
┌──────────────────────────────────────────────┐
│     ToolRegistry                             │
│     HashMap<String, Box<dyn Tool>>           │
│                                              │
│  ┌─────────────────┐  ┌──────────────────┐   │
│  │  默认工具 (5个)  │  │  动态注册工具     │   │
│  │  ReadFile        │  │  McpToolWrapper  │   │
│  │  WriteFile       │  │  CronTool       │   │
│  │  EditFile        │  │  SpawnTool      │   │
│  │  ListDir         │  │  ...            │   │
│  │  Exec            │  │                 │   │
│  └─────────────────┘  └──────────────────┘   │
│                                              │
│  execute(name, params) ──► 按名称查找并调用   │
│  get_definitions() ──► 导出 JSON Schema      │
└──────────────────────────────────────────────┘
                    │
                    ▼
          Provider::bind_tools()
```

## 关键类型

- **`Tool`** (trait) -- `name()`, `description()`, `parameters()`, `execute(ctx, params) -> ToolResult`, `to_definition()`
- **`ToolDefinition`** -- `name`, `description`, `parameters`（JSON Schema），用于 LLM function calling
- **`ToolContext`** -- `channel`, `chat_id` 执行上下文
- **`ToolError`** (enum) -- `Validation`, `Execution`, `NotFound`, `PermissionDenied`, `Timeout`, `Path`, `Io`
- **`ToolResult`** -- `Result<String, ToolError>` 类型别名
- **`ToolRegistry`** -- `HashMap<String, Box<dyn Tool>>`
  - `new(workspace, exec_config, restrict_to_workspace)` -- 创建并预注册所有默认工具
  - `register(tool)` -- 注册额外工具
  - `execute(ctx, name, params) -> ToolResult` -- 按名称分发执行
  - `get_definitions() -> Vec<ToolDefinition>` -- 返回所有工具定义供 LLM 绑定
- **`ReadFileTool`**, **`WriteFileTool`**, **`EditFileTool`**, **`ListDirTool`** -- 文件系统工具
- **`ExecTool`** -- Shell 命令执行，含安全防护（deny/allow 模式、工作空间限制）
- **`ExecToolOptions`** -- ExecTool 配置（deny/allow 模式、超时、工作空间、path_append）

## 内部依赖

utils, config
