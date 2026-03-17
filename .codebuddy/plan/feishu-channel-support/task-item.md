# 实施计划

- [ ] 1. 添加项目依赖和配置基础结构
   - 在 `crates/channels/Cargo.toml` 中添加 `feishu-sdk v0.1.2`、`thiserror`、`serde`、`async_trait` 等必要依赖
   - 创建 `FeishuConfig` 结构体，包含 `app_id`、`app_secret` 和可选的 `allow_from` 字段，实现 `serde` 序列化/反序列化
   - 实现配置验证逻辑，确保 `app_id` 和 `app_secret` 不为空
   - _需求：5、9_

- [ ] 2. 定义错误类型和基础数据结构
   - 使用 `thiserror` 在 crate 级别定义 `ChannelError` 枚举，包含 `InvalidConfig`、`StartFailed`、`StopFailed`、`SendFailed` 等变体
   - 定义 `FeishuChannel` 结构体，包含配置、运行状态和消息上下文等必要字段
   - 使用 `Arc` 和 `RwLock` 实现线程安全的状态管理
   - 实现 `Clone` trait 以支持通道实例的克隆
   - _需求：1、6_

- [ ] 3. 实现消息上下文管理
   - 使用 `Arc<RwLock<HashMap>>` 实现线程安全的消息上下文存储
   - 实现通过聊天 ID 检索原始消息的功能
   - 实现消息上下文的保存和清理逻辑
   - _需求：7_

- [ ] 4. 实现 WebSocket 连接管理
   - 使用 WebSocket 协议连接飞书服务器
   - 实现后台任务持续监听消息
   - 实现自动重连机制，包括重连次数限制和间隔配置
   - 实现优雅关闭 WebSocket 连接和取消后台任务
   - 添加 `info!` 级别的启动和停止日志
   - _需求：2_

- [ ] 5. 实现消息接收与处理逻辑
   - 解析飞书 WebSocket 消息，提取文本内容、发送者 ID 和聊天 ID
   - 实现空消息和非文本消息的过滤逻辑，添加 `warn!` 日志
   - 创建 `InboundMessage` 实例并通过通道发送到消息处理器
   - 实现白名单验证逻辑，检查发送者 ID 是否在允许列表中
   - 将原始消息保存到上下文中，使用聊天 ID 作为键
   - 添加消息接收的 `info!` 日志
   - _需求：3_

- [ ] 6. 实现消息发送功能
   - 通过飞书 HTTP API 发送消息
   - 使用 Markdown 格式化消息内容
   - 从上下文中获取原始消息以提取回复所需的参数
   - 处理上下文中找不到消息的情况，返回 `ChannelError::SendFailed` 错误
   - 添加消息发送的 `debug!` 日志，包含目标聊天和消息内容
   - _需求：4_

- [ ] 7. 实现 Channel trait 的所有必需方法
   - 实现 `start` 方法，建立 WebSocket 连接并启动后台任务
   - 实现 `stop` 方法，优雅关闭连接并清理资源
   - 实现 `send` 方法，发送消息到飞书
   - 实现 `is_running` 方法，返回通道运行状态
   - 实现 `name` 方法，返回 "feishu" 作为通道名称
   - 使用 `async_trait` 宏标记 trait 实现
   - _需求：1_

- [ ] 8. 添加完整的日志记录
   - 在所有关键操作处添加适当的日志级别（`info!`、`warn!`、`error!`、`debug!`）
   - 确保错误日志包含详细的错误堆栈信息
   - 确保忽略消息的日志说明忽略原因
   - _需求：8_

- [ ] 9. 编写单元测试
   - 创建 `crates/channels/src/feishu/tests.rs` 测试模块
   - 编写配置验证逻辑的测试用例
   - 编写消息处理逻辑的测试用例
   - 编写错误处理逻辑的测试用例
   - 编写权限检查逻辑的测试用例
   - 使用描述性的测试函数名称
   - 在 `mod.rs` 末尾通过 `#[cfg(test)] mod tests;` 引入测试模块
   - _需求：10_

- [ ] 10. 集成到 ChannelManager
   - 在 `ChannelManager` 中注册飞书通道
   - 确保飞书通道与其他通道保持一致的接口和行为
   - 验证通道名称为 "feishu"
   - _需求：11_

- [ ] 11. 代码风格检查和优化
   - 确保使用 Rust 2024 版本特性
   - 使用 `let chains` 特性合并嵌套的 `if` 语句
   - 确保所有 `serde` 配置使用 `#[serde(rename_all="camelCase")]`
   - 遵循项目现有的代码风格规范
   - _需求：12_
