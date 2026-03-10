# 需求文档

## 引言

本需求旨在为 `Tool` trait 的 `execute` 方法添加上下文参数支持。当前 `execute` 方法仅接收 `params: serde_json::Value` 参数，缺乏对调用上下文（如通道名称、聊天标识）的访问能力。通过引入 `ToolContext` 类型，工具实现可以获取当前请求的来源信息，从而支持更多场景化功能（如按通道区分行为、发送回复消息等）。

## 需求

### 需求 1：定义 ToolContext 类型

**用户故事：** 作为工具开发者，我希望有一个统一的上下文类型来携带请求来源信息，以便在工具执行时能够访问这些信息。

#### 验收标准

1. WHEN 定义 ToolContext 类型 THEN 系统 SHALL 提供一个包含 `channel` 和 `chat_id` 字段的结构体
2. IF ToolContext 中的字段使用引用类型 THEN 系统 SHALL 确保生命周期参数正确标注
3. WHEN ToolContext 被创建 THEN 系统 SHALL 提供 `new` 构造函数，接受 `channel` 和 `chat_id` 参数
4. WHEN 需要访问 ToolContext 字段 THEN 系统 SHALL 提供只读的 getter 方法

### 需求 2：拓展 Tool trait execute 方法签名

**用户故事：** 作为工具开发者，我希望 execute 方法能够接收上下文参数，以便工具可以根据调用来源进行差异化处理。

#### 验收标准

1. WHEN 修改 execute 方法签名 THEN 系统 SHALL 添加 `ctx: &ToolContext` 参数
2. WHEN execute 方法被调用 THEN 系统 SHALL 传递包含当前请求 channel 和 chat_id 的 ToolContext 引用
3. IF 工具实现不需要使用上下文 THEN 系统 SHALL 允许使用下划线 `_` 忽略该参数

### 需求 3：更新 ToolRegistry execute 方法

**用户故事：** 作为工具使用者，我希望 ToolRegistry 的 execute 方法能够传递上下文信息，以便工具执行时能够获取来源信息。

#### 验收标准

1. WHEN 修改 ToolRegistry::execute 方法 THEN 系统 SHALL 添加 `ctx: &ToolContext` 参数
2. WHEN ToolRegistry::execute 调用内部工具的 execute 方法 THEN 系统 SHALL 传递 ctx 参数
3. IF ToolRegistry::execute 被调用时缺少上下文 THEN 系统 SHALL 返回编译错误

### 需求 4：更新现有 Tool 实现

**用户故事：** 作为开发者，我希望所有现有的 Tool 实现能够兼容新的接口签名，以便系统保持编译通过。

#### 验收标准

1. WHEN 更新 ShellTool 的 execute 实现 THEN 系统 SHALL 接受 ToolContext 参数（可忽略）
2. WHEN 更新 ReadFileTool 的 execute 实现 THEN 系统 SHALL 接受 ToolContext 参数（可忽略）
3. WHEN 更新 WriteFileTool 的 execute 实现 THEN 系统 SHALL 接受 ToolContext 参数（可忽略）
4. WHEN 更新 EditFileTool 的 execute 实现 THEN 系统 SHALL 接受 ToolContext 参数（可忽略）
5. WHEN 更新 ListDirTool 的 execute 实现 THEN 系统 SHALL 接受 ToolContext 参数（可忽略）
6. WHEN 更新 CronTool 的 execute 实现 THEN 系统 SHALL 接受 ToolContext 参数（可忽略）

### 需求 5：更新 Agent 层调用

**用户故事：** 作为 Agent 模块使用者，我希望 Agent 能够正确构建并传递 ToolContext，以便工具执行时有完整的上下文信息。

#### 验收标准

1. WHEN Agent 的 ReAct 循环调用工具 THEN 系统 SHALL 从 InboundMessage 提取 channel 和 chat_id 构建 ToolContext
2. WHEN 调用 ToolRegistry::execute THEN 系统 SHALL 传递构建的 ToolContext 实例
3. IF InboundMessage 信息不可用 THEN 系统 SHALL 使用合理的默认值或返回错误
