# 需求文档：MCP 服务连接与工具包装器

## 引言

本功能旨在为 nanobot-rs 项目实现 MCP（Model Context Protocol）服务连接能力，参照 Python 版本的 `MCPToolWrapper` 实现，允许 AI Agent 通过统一的 Tool trait 接口调用 MCP 服务器提供的工具。

MCP 是一种标准化的协议，允许 AI 模型与外部工具和服务进行交互。通过实现 MCP 客户端支持，nanobot-rs 可以无缝集成各类 MCP 服务器提供的工具，扩展 Agent 的能力边界。

## 需求

### 需求 1：MCP 客户端核心模块

**用户故事：** 作为 nanobot 开发者，我希望有一个独立的 MCP 客户端模块，以便复用 MCP 连接和通信逻辑。

#### 验收标准

1. WHEN 创建 MCP 客户端模块 THEN 系统 SHALL 在 `crates/tools/src/` 下新增 `mcp.rs` 文件
2. WHEN 模块初始化 THEN 系统 SHALL 导出 `McpToolWrapper` 和 `connect_mcp_servers` 等公共 API
3. IF 项目依赖缺失 THEN 系统 SHALL 在 `Cargo.toml` 中添加 `rmcp` 或等效的 Rust MCP 客户端库依赖

### 需求 2：MCPToolWrapper 工具包装器

**用户故事：** 作为 AI Agent，我希望 MCP 服务器的工具能够以原生 nanobot Tool 的形式呈现，以便统一调用方式。

#### 验收标准

1. WHEN 创建 MCPToolWrapper THEN 系统 SHALL 实现 `Tool` trait
2. WHEN MCPToolWrapper 初始化 THEN 系统 SHALL 接收 session、server_name、tool_def 和 tool_timeout 参数
3. WHEN 调用 `name()` 方法 THEN 系统 SHALL 返回格式为 `mcp_{server_name}_{original_tool_name}` 的名称
4. WHEN 调用 `description()` 方法 THEN 系统 SHALL 返回 MCP 工具的描述信息，若描述为空则使用工具名称
5. WHEN 调用 `parameters()` 方法 THEN 系统 SHALL 返回 MCP 工具的 inputSchema，若无 schema 则返回空对象
6. WHEN 执行 `execute()` 方法 THEN 系统 SHALL 通过 MCP session 调用原始工具并返回结果
7. WHEN 工具执行超时 THEN 系统 SHALL 返回超时错误信息，而非阻塞或 panic

### 需求 3：Stdio 传输连接支持

**用户故事：** 作为用户，我希望能够通过命令行启动并连接 MCP 服务器，以便使用本地 MCP 工具。

#### 验收标准

1. WHEN 配置了 `command` 字段 THEN 系统 SHALL 通过 stdio 方式启动 MCP 服务器进程
2. WHEN 启动 MCP 服务器 THEN 系统 SHALL 支持传递 `command`、`args` 和 `env` 参数
3. WHEN 连接建立 THEN 系统 SHALL 创建 MCP ClientSession 并完成初始化握手
4. IF 进程启动失败或连接失败 THEN 系统 SHALL 记录错误日志并跳过该服务器

### 需求 4：HTTP/SSE 传输连接支持

**用户故事：** 作为用户，我希望能够通过 HTTP 连接远程 MCP 服务器，以便使用云端 MCP 工具。

#### 验收标准

1. WHEN 配置了 `url` 字段 THEN 系统 SHALL 通过 HTTP/SSE 方式连接 MCP 服务器
2. WHEN 创建 HTTP 连接 THEN 系统 SHALL 支持自定义 `headers` 配置
3. WHEN 创建 HTTP 客户端 THEN 系统 SHALL 禁用默认超时以支持工具级别的超时控制
4. WHEN 连接建立 THEN 系统 SHALL 创建 MCP ClientSession 并完成初始化握手
5. IF HTTP 连接失败 THEN 系统 SHALL 记录错误日志并跳过该服务器

### 需求 5：MCP 服务器连接管理

**用户故事：** 作为用户，我希望能够同时连接多个 MCP 服务器，以便整合多个工具来源。

#### 验收标准

1. WHEN 调用 `connect_mcp_servers` 函数 THEN 系统 SHALL 遍历所有配置的 MCP 服务器
2. WHEN 成功连接 MCP 服务器 THEN 系统 SHALL 列出该服务器的所有工具并注册到 ToolRegistry
3. WHEN 注册工具 THEN 系统 SHALL 为每个工具创建 MCPToolWrapper 包装器
4. WHEN 服务器连接成功 THEN 系统 SHALL 记录 INFO 级别日志，包含服务器名称和工具数量
5. WHEN 服务器连接失败 THEN 系统 SHALL 记录 ERROR 级别日志，但继续处理其他服务器
6. WHEN 服务器缺少 command 和 url 配置 THEN 系统 SHALL 记录 WARNING 日志并跳过

### 需求 6：配置结构定义

**用户故事：** 作为用户，我希望通过配置文件定义 MCP 服务器连接参数，以便灵活管理工具来源。

#### 验收标准

1. WHEN 定义配置结构 THEN 系统 SHALL 创建 `McpServerConfig` 枚举，区分 `Stdio` 和 `Http` 两种变体
2. WHEN 配置 Stdio 类型 THEN 系统 SHALL 包含 `command`、`args` 和 `env` 字段
3. WHEN 配置 Http 类型 THEN 系统 SHALL 包含 `url` 和 `headers` 字段
4. WHEN 定义 MCP 服务器配置 THEN 系统 SHALL 在枚举外层包含公共的 `tool_timeout` 字段
5. WHEN 未指定 tool_timeout THEN 系统 SHALL 使用默认值 30 秒
6. WHEN 配置被解析 THEN 系统 SHALL 支持从 TOML/YAML 配置文件加载 MCP 服务器配置
7. WHEN 序列化配置 THEN 系统 SHALL 确保两种配置类型可被正确序列化和反序列化

### 需求 7：生命周期管理

**用户故事：** 作为系统，我希望 MCP 连接能够正确管理生命周期，以便资源被正确释放。

#### 验收标准

1. WHEN 创建 MCP 连接 THEN 系统 SHALL 支持异步上下文管理（类似 Python 的 AsyncExitStack）
2. WHEN 应用关闭 THEN 系统 SHALL 正确关闭所有 MCP session 和相关资源
3. WHEN 连接断开 THEN 系统 SHALL 避免资源泄漏

### 需求 8：错误处理

**用户故事：** 作为用户，我希望 MCP 相关错误能够被妥善处理和报告，以便快速定位问题。

#### 验收标准

1. WHEN 工具执行超时 THEN 系统 SHALL 返回格式为 `(MCP tool call timed out after {timeout}s)` 的信息
2. WHEN 工具返回空内容 THEN 系统 SHALL 返回 `(no output)`
3. WHEN 工具返回多块内容 THEN 系统 SHALL 以换行符连接所有文本块
4. WHEN 非文本内容块被返回 THEN 系统 SHALL 将其转换为字符串形式
