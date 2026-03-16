# 需求文档

## 引言

本需求文档描述为 Rust 版 `AgentLoop` 新增 `process_system_message` 私有方法的实现需求。该方法用于处理 `channel=system` 的系统消息，参考 Python 版 `_process_message` 函数中处理系统消息的逻辑实现。

系统消息是一种特殊的消息类型，其 `chat_id` 字段包含实际的目标通道和聊天 ID（格式为 "channel:chat_id"），需要在处理时进行解析并路由到正确的目标。

## 需求

### 需求 1：解析系统消息的目标路由信息

**用户故事：** 作为 AgentLoop 开发者，我希望能够从系统消息的 chat_id 字段中解析出实际的目标通道和聊天 ID，以便正确路由系统消息。

#### 验收标准

1. WHEN 系统消息的 chat_id 包含冒号分隔符 THEN 系统 SHALL 将其分割为 channel 和 chat_id 两部分
2. WHEN 系统消息的 chat_id 不包含冒号分隔符 THEN 系统 SHALL 使用默认值 "cli" 作为 channel，原始 chat_id 作为 chat_id
3. IF 解析成功 THEN 系统 SHALL 使用 "{channel}:{chat_id}" 格式构建会话 key

### 需求 2：处理系统消息并返回响应

**用户故事：** 作为 AgentLoop 开发者，我希望有一个专门的私有方法来处理系统消息，以便与普通消息处理逻辑分离，保持代码清晰。

#### 验收标准

1. WHEN 收到 channel=system 的消息 THEN 系统 SHALL 调用 `process_system_message` 私有方法处理该消息
2. WHEN `process_system_message` 被调用 THEN 系统 SHALL 解析目标路由信息、获取或创建会话、设置工具上下文、构建消息历史
3. WHEN 消息处理完成 THEN 系统 SHALL 返回 OutboundMessage，其 channel 和 chat_id 为解析后的目标值
4. WHEN 消息处理完成 THEN 系统 SHALL 保存会话状态以持久化本次对话

### 需求 3：集成到 process_message 方法

**用户故事：** 作为 AgentLoop 开发者，我希望 `process_message` 方法能够自动识别系统消息并路由到 `process_system_message`，以便保持现有的调用接口不变。

#### 验收标准

1. IF 入站消息的 channel 字段为 "system" THEN 系统 SHALL 调用 `process_system_message` 方法
2. IF 入站消息的 channel 字段不为 "system" THEN 系统 SHALL 继续执行现有的普通消息处理逻辑
3. WHEN 系统消息处理完成 THEN 系统 SHALL 返回解析后的目标通道的 OutboundMessage
