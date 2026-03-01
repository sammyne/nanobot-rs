# 需求文档

## 引言

本需求文档描述如何将 Rust 版 nanobot CLI 的 agent 子命令的多轮对话逻辑，从当前直接调用 Provider 的方式，调整为使用 `AgentLoop` 实现，以对齐 Python 版本的设计架构和功能特性。

当前 Rust 版的 `run_interactive` 函数存在以下问题：
1. 直接使用 `provider.chat(&messages)` 调用 LLM，绕过了 `AgentLoop`
2. 手动管理 `messages` 列表，与 `AgentLoop` 的职责重叠
3. 缺少消息总线（MessageBus）机制
4. 缺少进度回调显示（thinking 状态提示）
5. 输入处理较简单，不支持历史记录等功能

本次需求的目标是重构 Rust 版的多轮对话逻辑，使其与 Python 版保持一致的架构设计。

## 需求

### 需求 1：消息总线架构与交互式对话流程

**用户故事：** 作为一名开发者，我希望 Rust 版的交互式对话使用与 Python 版一致的消息总线架构，以便保持代码架构的统一性和可维护性。

#### 验收标准

1. WHEN 启动交互式对话 THEN 系统 SHALL 创建 MessageBus 实例用于消息路由
2. WHEN 创建 AgentLoop THEN 系统 SHALL 将 MessageBus 实例传递给 AgentLoop 构造函数
3. WHEN 启动交互式会话 THEN 系统 SHALL 调用 `agent_loop.run()` 启动后台任务处理消息循环
4. WHEN 用户发送消息 THEN 系统 SHALL 通过 `bus.publish_inbound(InboundMessage)` 发布用户消息到消息总线
5. WHEN AgentLoop 处理消息 THEN 系统 SHALL 在后台任务中自动消费入站消息并处理
6. WHEN AgentLoop 产生响应 THEN 系统 SHALL 通过消息总线的出站队列发送响应
7. WHEN CLI 接收响应 THEN 系统 SHALL 通过 `bus.consume_outbound()` 异步消费出站消息
8. IF 出站消息包含进度元数据 THEN 系统 SHALL 根据配置显示进度提示或工具调用提示
9. IF 出站消息是最终响应 THEN 系统 SHALL 渲染并显示完整的响应内容
10. WHEN 退出交互模式 THEN 系统 SHALL 停止后台任务并清理 MessageBus 资源

### 需求 2：进度显示与用户反馈

**用户故事：** 作为一名用户，我希望在等待 AI 响应时能看到进度提示，以便了解系统正在工作。

#### 验收标准

1. WHEN AgentLoop 正在处理消息 THEN 系统 SHALL 显示 "nanobot is thinking..." 或类似的进度提示
2. WHEN 收到 LLM 响应 THEN 系统 SHALL 清除进度提示并显示最终响应
3. IF 配置了进度回调 THEN 系统 SHALL 通过消息总线发送进度更新
4. IF 出站消息的 metadata 包含 `_progress` 字段 THEN 系统 SHALL 显示进度内容
5. IF 出站消息的 metadata 包含 `_tool_hint` 字段 THEN 系统 SHALL 根据配置决定是否显示工具调用提示

### 需求 3：退出命令处理

**用户故事：** 作为一名用户，我希望能够通过简单的命令退出交互模式，以便灵活控制对话流程。

#### 验收标准

1. WHEN 用户输入 "exit"、"quit"、"/exit"、"/quit" 或 ":q" THEN 系统 SHALL 退出交互模式
2. WHEN 用户输入 Ctrl+C 或 Ctrl+D THEN 系统 SHALL 优雅退出并显示告别信息
3. WHEN 退出交互模式 THEN 系统 SHALL 恢复终端到原始状态

### 需求 4：输入处理优化

**用户故事：** 作为一名用户，我希望输入体验更加友好，包括支持历史记录和干净的用户界面。

#### 验收标准

1. WHEN 用户按下上箭头 THEN 系统 SHALL 显示上一条输入记录
2. WHEN 用户粘贴多行内容 THEN 系统 SHALL 正确处理而不产生显示问题
3. IF 输入为空 THEN 系统 SHALL 不发送消息并继续等待输入
4. WHEN AgentLoop 正在处理响应 THEN 系统 SHALL 清空终端待处理的输入缓冲区

### 需求 5：错误处理与恢复

**用户故事：** 作为一名用户，我希望在发生错误时能得到清晰的提示，并能继续对话而不是直接退出。

#### 验收标准

1. WHEN LLM 调用失败 THEN 系统 SHALL 显示错误信息并允许用户继续对话
2. WHEN 网络超时 THEN 系统 SHALL 显示超时提示并允许重试
3. IF AgentLoop 初始化失败 THEN 系统 SHALL 显示配置错误提示并退出

### 需求 6：会话上下文管理

**用户故事：** 作为一名开发者，我希望 AgentLoop 负责管理消息历史和会话上下文，以便 CLI 层保持简洁。

#### 验收标准

1. WHEN CLI 发送消息到消息总线 THEN 系统 SHALL 不再手动维护 messages 列表
2. IF AgentLoop 支持会话上下文 THEN 系统 SHALL 在同一会话 ID 中保持对话连贯性
3. WHEN 指定 session_id THEN 系统 SHALL 将其解析为 channel 和 chat_id 用于消息路由
