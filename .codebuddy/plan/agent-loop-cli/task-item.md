# 实施计划

## 前置依赖说明
本实施计划依赖于 `agent-loop-rs` 需求中定义的 MessageBus 和 AgentLoop 异步 run() 方法的实现。在执行本计划前，需确认这些基础设施已就绪。

- [ ] 1. 实现 MessageBus 消息总线基础设施
   - 在 `crates/agent/src/` 下创建 `bus.rs` 模块
   - 实现 `InboundMessage` 和 `OutboundMessage` 结构体（包含 channel、chat_id、content、metadata 字段）
   - 实现 `MessageBus` 结构体，包含入站和出站异步队列（使用 tokio::sync::mpsc）
   - 实现 `publish_inbound()` 和 `publish_outbound()` 方法用于发布消息
   - 实现 `consume_outbound()` 方法用于消费出站消息
   - _需求：1.1、1.2、1.4、1.6、1.7_

- [ ] 2. 为 AgentLoop 添加异步 run() 方法
   - 在 `AgentLoop` 中添加 `MessageBus` 字段
   - 实现 `run()` 异步方法，启动后台任务消费入站消息并处理
   - 实现 `stop()` 方法用于优雅停止后台任务
   - 修改 `process_direct()` 方法支持 `on_progress` 回调参数
   - 处理进度消息时通过消息总线发送 `_progress` 元数据
   - _需求：1.3、1.5、1.6、2.3、2.4_

- [ ] 3. 重构 run_interactive 函数使用 AgentLoop 和 MessageBus
   - 创建 MessageBus 实例并传递给 AgentLoop
   - 移除手动管理的 `messages` 列表，改由 AgentLoop 管理
   - 启动 `agent_loop.run()` 后台任务
   - 通过 `bus.publish_inbound()` 发布用户消息
   - 实现 `_consume_outbound()` 异步任务消费响应消息
   - _需求：1.1、1.3、1.4、1.7、6.1_

- [ ] 4. 实现进度显示与 thinking 状态提示
   - 使用 `console.status()` 或类似机制显示 "nanobot is thinking..." 动画
   - 处理出站消息中的 `_progress` 元数据显示进度内容
   - 处理 `_tool_hint` 元数据显示工具调用提示（根据配置决定是否显示）
   - 响应完成后清除进度提示并显示最终响应
   - _需求：2.1、2.2、2.4、2.5_

- [ ] 5. 优化输入处理与用户体验
   - 使用 `rustyline` 或 `reedline` crate 实现 readline 功能
   - 支持历史记录（上箭头导航）
   - 实现输入缓冲区刷新（参考 Python 版 `_flush_pending_tty_input`）
   - 处理空输入跳过逻辑
   - _需求：4.1、4.2、4.3、4.4_

- [ ] 6. 实现优雅退出与信号处理
   - 支持多种退出命令：exit、quit、/exit、/quit、:q
   - 处理 Ctrl+C (SIGINT) 和 Ctrl+D (EOF) 信号
   - 退出时恢复终端原始状态
   - 停止后台任务并清理 MessageBus 资源
   - 显示告别信息
   - _需求：1.10、3.1、3.2、3.3_

- [ ] 7. 完善错误处理与恢复机制
   - LLM 调用失败时显示错误信息但允许继续对话
   - 网络超时时显示提示并允许重试
   - AgentLoop 初始化失败时显示配置错误提示并退出
   - _需求：5.1、5.2、5.3_

- [ ] 8. 实现会话上下文管理
   - 解析 session_id 参数为 channel 和 chat_id
   - AgentLoop 根据会话 ID 维护对话上下文
   - 确保同一会话中对话连贯性
   - _需求：6.2、6.3_

- [ ] 9. 编写单元测试和集成测试
   - 为 MessageBus 编写单元测试（消息发布/消费）
   - 为 AgentLoop 的 run() 方法编写异步测试
   - 为交互式对话流程编写集成测试
   - 测试退出命令和信号处理
   - _需求：全部验收标准_
