# 需求文档

## 引言

本功能旨在将现有的 MCP（Model Context Protocol）集成到 AgentLoop 核心处理引擎中，使 Agent 能够通过统一的 Tool trait 接口调用 MCP 服务器提供的工具。通过此集成，Agent 可以动态访问外部 MCP 服务器的各种工具（如文件操作、系统命令、网络服务等），从而扩展 Agent 的能力边界。

当前 MCP 客户端功能已在 `crates/mcp` 模块中实现，包括 MCP 服务器的连接管理（支持 Stdio 和 HTTP 两种方式）以及工具包装器 `McpToolWrapper`。本需求的目标是将这些功能无缝集成到 AgentLoop 的初始化流程中，并在配置系统中添加对 MCP 服务器的配置支持。

## 需求

### 需求 1：在 Config 的 ToolsConfig 中添加 MCP 服务器配置字段

**用户故事：** 作为一名系统管理员，我希望在配置文件的 `tools.mcp_servers` 字段中定义 MCP 服务器列表，以便 Agent 启动时能够自动连接并注册这些服务器提供的工具。

#### 验收标准

1. WHEN 配置文件的 `tools` 节点包含 `mcp_servers` 字段时，THEN 系统 SHALL 解析并验证该字段
2. IF `tools.mcp_servers` 字段不存在，THEN 系统 SHALL 使用空配置（不连接任何 MCP 服务器）
3. WHEN `tools.mcp_servers` 字段存在时，THEN 系统 SHALL 支持多个 MCP 服务器的配置，每个服务器由唯一的名称标识
4. WHEN 配置中某个 MCP 服务器配置无效时，THEN 系统 SHALL 返回验证错误并拒绝启动
5. WHEN MCP 服务器配置类型为 Stdio 时，THEN 系统 SHALL 包含 `command`、`args`、`env` 字段
6. WHEN MCP 服务器配置类型为 Http 时，THEN 系统 SHALL 包含 `url`、`headers`、`tool_timeout` 字段

### 需求 2：在 AgentLoop 初始化时自动连接并注册 MCP 工具

**用户故事：** 作为一名开发者，我希望 AgentLoop 在创建时能够自动连接配置的 MCP 服务器并注册所有可用工具，以便无需手动干预即可使用 MCP 工具。

#### 验收标准

1. WHEN AgentLoop::new() 被调用且 `config.tools.mcp_servers` 不为空时，THEN 系统 SHALL 调用 `mcp::wrapper::connect()` 函数连接所有配置的 MCP 服务器
2. WHEN MCP 连接成功时，THEN 系统 SHALL 将返回的所有工具包装器注册到 ToolRegistry 中
3. WHEN MCP 连接失败时，THEN 系统 SHALL 返回包含详细错误信息的 Err
4. WHEN 工具注册成功时，THEN 系统 SHALL 为每个工具打印日志，显示服务器名称和工具名称
5. WHEN 注册的 MCP 工具名称格式 SHALL 为 `mcp_{server_name}_{original_tool_name}`
6. WHEN 注册工具后，THEN 系统 SHALL 将所有工具定义（包括 MCP 工具）绑定到 Provider，使 LLM 可以调用这些工具

### 需求 3：在配置文件 Schema 中添加 MCP 配置示例

**用户故事：** 作为一名用户，我希望配置文件包含清晰的 MCP 配置示例，以便能够快速了解如何配置 MCP 服务器。

#### 验收标准

1. WHEN 用户查看配置文件时，THEN 系统 SHALL 包含至少一个 Stdio 类型 MCP 服务器的配置示例
2. WHEN 用户查看配置文件时，THEN 系统 SHALL 包含至少一个 HTTP 类型 MCP 服务器的配置示例
3. WHEN 配置示例 SHALL 包含所有必需字段和可选字段的说明注释
4. WHEN 配置示例 SHALL 使用常见的 MCP 服务器（如 filesystem、sqlite 等）作为示例

### 需求 4：MCP 连接错误处理和日志记录

**用户故事：** 作为一名运维人员，我希望系统在 MCP 连接失败时能够提供清晰的错误信息和日志，以便快速诊断和解决问题。

#### 验收标准

1. WHEN MCP 服务器连接失败时，THEN 系统 SHALL 记录详细的错误日志，包括服务器名称、错误类型和错误信息
2. WHEN MCP 工具调用超时时，THEN 系统 SHALL 记录警告日志，显示工具名称和超时时间
3. WHEN MCP 工具调用失败时，THEN 系统 SHALL 记录错误日志，包含工具名称和失败原因
4. WHEN AgentLoop 初始化时，THEN 系统 SHALL 记录连接的 MCP 服务器数量和注册的工具总数

### 需求 5：确保 MCP 工具包装器的生命周期管理

**用户故事：** 作为一名开发者，我希望 MCP 连接在 AgentLoop 销毁时能够正确关闭，以便释放系统资源。

#### 验收标准

1. WHEN AgentLoop 被销毁时，THEN 系统 SHALL 通过 McpToolWrapper 的 Drop 实现正确关闭所有 MCP 连接
2. WHEN McpToolWrapper 的 Drop 被调用且 Arc 强引用计数为 1 时，THEN 系统 SHALL 取消 MCP 连接的 cancellation token
3. WHEN MCP 连接关闭时，THEN 系统 SHALL 记录日志，显示被关闭的服务器名称