# 实施计划

## 任务清单

- [ ] 1. 定义 MCP 服务器配置结构体
  - 在 `crates/config` 中定义 `McpServerConfig`、`StdioConfig` 和 `HttpConfig` 结构体
  - 使用 `serde` 实现配置的序列化和反序列化
  - 实现配置验证逻辑（确保必需字段存在且有效）
  - _需求：1.1、1.5、1.6_

- [ ] 2. 在 ToolsConfig 中添加 MCP 服务器配置字段
  - 修改 `ToolsConfig` 结构体，添加 `mcp_servers: HashMap<String, McpServerConfig>` 字段
  - 设置该字段为可选，支持空配置
  - 更新配置解析逻辑以处理新字段
  - _需求：1.1、1.2、1.3、1.4_

- [ ] 3. 更新示例配置文件
  - 在配置文件示例中添加 `tools.mcp_servers` 节点
  - 包含 Stdio 类型示例（filesystem 服务器）
  - 包含 HTTP 类型示例
  - 为所有字段添加中文说明注释
  - _需求：3.1、3.2、3.3、3.4_

- [ ] 4. 在 AgentLoop 中添加 MCP 连接和注册逻辑
  - 在 `AgentLoop::new()` 方法中添加 MCP 连接逻辑
  - 遍历 `config.tools.mcp_servers`，为每个服务器调用 `mcp::wrapper::connect()`
  - 将返回的 `McpToolWrapper` 注册到 `ToolRegistry`，工具名称格式为 `mcp_{server_name}_{original_tool_name}`
  - 在注册成功时打印日志
  - _需求：2.1、2.2、2.4、2.5_

- [ ] 5. 实现 MCP 连接错误处理
  - 在 MCP 连接失败时返回包含详细错误信息的 Err
  - 记录连接失败的详细日志（服务器名称、错误类型、错误信息）
  - 确保单个服务器连接失败不影响其他服务器
  - _需求：2.3、4.1_

- [ ] 6. 将 MCP 工具绑定到 Provider
  - 在工具注册完成后，将所有工具（包括 MCP 工具）绑定到 Provider
  - 确保 LLM 可以通过 Provider 调用 MCP 工具
  - _需求：2.6_

- [ ] 7. 添加 MCP 工具调用日志记录
  - 在 `McpToolWrapper::execute()` 中添加超时警告日志
  - 在工具调用失败时记录错误日志（工具名称、失败原因）
  - _需求：4.2、4.3_

- [ ] 8. 记录 AgentLoop 初始化统计信息
  - 在 AgentLoop 初始化完成后记录连接的 MCP 服务器数量
  - 记录注册的工具总数
  - _需求：4.4_

- [ ] 9. 验证 MCP 工具生命周期管理
  - 确认 `McpToolWrapper` 的 Drop 实现（在 crates/mcp 中）
  - 验证在 Arc 强引用计数为 1 时正确取消 cancellation token
  - 在 MCP 连接关闭时添加日志记录（服务器名称）
  - _需求：5.1、5.2、5.3_

- [ ] 10. 编写集成测试
  - 创建测试配置文件，包含多个 MCP 服务器配置
  - 测试 AgentLoop 初始化时的 MCP 连接和工具注册
  - 测试 MCP 工具的正确调用
  - 测试错误场景（无效配置、连接失败等）
  - _需求：1.4、2.3、4.1、5.1_
