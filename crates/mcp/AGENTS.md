# mcp crate

MCP 客户端，将 MCP 服务器工具桥接为统一 Tool 接口。

## 关键类型

- **`McpToolWrapper`** -- 实现 `Tool` trait，包装 MCP 服务器工具，名称格式 `mcp_{server}_{tool}`，支持超时控制和结果格式化
- **`McpError`** (enum) -- `ProcessSpawnFailed`, `HttpConnectionFailed`, `InitializationFailed`, `ToolListFailed`, `InvalidConfig`, `SchemaParseFailed`
- **`connect(configs: HashMap<String, McpServerConfig>) -> Result<Vec<McpToolWrapper>>`** -- 连接所有配置的 MCP 服务器（Stdio 或 HTTP），返回工具包装器列表

## 内部依赖

config, tools
