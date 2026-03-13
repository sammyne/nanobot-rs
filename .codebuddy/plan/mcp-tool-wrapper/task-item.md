# 实施计划：MCP 服务连接与工具包装器

- [ ] 1. 添加 MCP 客户端库依赖和模块结构
   - 在 `crates/tools/Cargo.toml` 中添加 `rmcp` 或等效的 Rust MCP 客户端库依赖
   - 在 `crates/tools/src/` 下创建 `mcp.rs` 模块文件
   - 在 `crates/tools/src/lib.rs` 中导出 `mcp` 模块
   - _需求：1.1、1.3_

- [ ] 2. 定义配置结构
   - 创建 `McpServerConfig` 枚举，包含 `Stdio` 和 `Http` 两个变体
   - `Stdio` 变体包含 `command`、`args`、`env` 字段
   - `Http` 变体包含 `url`、`headers` 字段
   - 在配置结构外层定义公共的 `tool_timeout` 字段，默认值为 30 秒
   - 实现 `Serialize` 和 `Deserialize` trait，支持 TOML/YAML 配置文件加载
   - _需求：6.1、6.2、6.3、6.4、6.5、6.6、6.7_

- [ ] 3. 实现 MCPToolWrapper 工具包装器
   - 定义 `McpToolWrapper` 结构体，包含 session、server_name、tool_def、tool_timeout 字段
   - 实现 `Tool` trait 的 `name()` 方法，返回 `mcp_{server_name}_{original_tool_name}` 格式
   - 实现 `Tool` trait 的 `description()` 方法，返回 MCP 工具描述或工具名称
   - 实现 `Tool` trait 的 `parameters()` 方法，返回 inputSchema 或空对象
   - 实现 `Tool` trait 的 `execute()` 方法，通过 MCP session 调用工具并处理超时
   - _需求：2.1、2.2、2.3、2.4、2.5、2.6、2.7_

- [ ] 4. 实现 Stdio 传输连接
   - 实现 Stdio 方式的 MCP 服务器进程启动逻辑
   - 支持传递 `command`、`args`、`env` 参数启动子进程
   - 创建 MCP ClientSession 并完成初始化握手
   - 添加错误处理，进程启动失败时记录错误日志并跳过
   - _需求：3.1、3.2、3.3、3.4_

- [ ] 5. 实现 HTTP/SSE 传输连接
   - 实现 HTTP/SSE 方式的 MCP 服务器连接逻辑
   - 支持自定义 `headers` 配置
   - 创建禁用默认超时的 HTTP 客户端
   - 创建 MCP ClientSession 并完成初始化握手
   - 添加错误处理，连接失败时记录错误日志并跳过
   - _需求：4.1、4.2、4.3、4.4、4.5_

- [ ] 6. 实现服务器连接管理函数
   - 实现 `connect_mcp_servers` 异步函数
   - 遍历所有配置的 MCP 服务器，依次建立连接
   - 成功连接后列出服务器工具并注册到 ToolRegistry
   - 为每个工具创建 MCPToolWrapper 包装器
   - 记录连接成功的 INFO 日志和失败的 ERROR 日志
   - _需求：5.1、5.2、5.3、5.4、5.5、5.6_

- [ ] 7. 实现生命周期管理
   - 使用 `tokio::task::JoinSet` 或类似机制管理 MCP 连接的生命周期
   - 实现应用关闭时正确关闭所有 MCP session 和相关资源
   - 确保连接断开时无资源泄漏
   - _需求：7.1、7.2、7.3_

- [ ] 8. 完善错误处理
   - 实现工具执行超时时返回 `(MCP tool call timed out after {timeout}s)` 信息
   - 实现工具返回空内容时返回 `(no output)`
   - 实现多块内容以换行符连接，非文本内容块转换为字符串
   - _需求：8.1、8.2、8.3、8.4_

- [ ] 9. 添加单元测试
   - 为 `McpServerConfig` 配置结构的序列化和反序列化编写测试
   - 为 `MCPToolWrapper` 的名称生成和参数处理编写测试
   - 为错误处理逻辑编写测试
   - _需求：1.1、2.3、2.5、6.7、8.1、8.2_

- [ ] 10. 集成测试和文档更新
   - 编写集成测试，测试 Stdio 和 HTTP 两种连接方式
   - 测试多服务器连接和工具注册流程
   - 更新 `crates/tools/README.md` 或相关文档，说明 MCP 功能的使用方法
   - _需求：3.1、4.1、5.1_