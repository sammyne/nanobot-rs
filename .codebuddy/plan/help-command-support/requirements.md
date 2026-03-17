# 需求文档

## 引言

本需求文档描述了在 Rust 版本的 `AgentLoop` 中添加 `/help` 命令支持的功能。该功能参考 Python 版本的实现，为用户提供查看可用命令的能力，提升用户体验和系统可用性。

目前 Python 版本的 `process_message` 方法已经支持 `/help` 和 `/new` 命令，而 Rust 版本尚未实现此功能。本需求将首先实现 `/help` 命令，为后续扩展其他命令奠定基础。

## 需求

### 需求 1：命令架构设计

**用户故事：** 作为一名开发者，我希望系统采用统一的命令处理架构，以便代码结构清晰且易于维护。

#### 验收标准

1. WHEN 用户发送以 `/` 开头的消息 THEN 系统 SHALL 将其识别为命令而非普通对话内容
2. WHEN 系统识别到命令 THEN 系统 SHALL 调用 `AgentLoop` 的 `handle_cmd` 方法进行统一处理
3. IF 消息不以 `/` 开头 THEN 系统 SHALL 按照普通消息流程处理（调用 LLM 等）
4. WHEN 实现 `handle_cmd` 方法 THEN 系统 SHALL 采用清晰的结构，便于添加新命令

### 需求 2：`/help` 命令识别

**用户故事：** 作为一名用户，我希望系统能够准确识别我输入的 `/help` 命令，以便获取帮助信息。

#### 验收标准

1. WHEN 用户发送内容为 `/help` 的消息 THEN 系统 SHALL 通过 `handle_cmd` 方法识别并处理该命令
2. WHEN 用户发送内容为 `/HELP` 或 `/Help` 的消息 THEN 系统 SHALL 将其识别为 `/help` 命令（不区分大小写）
3. WHEN 用户发送内容为 `/help `（带空格）的消息 THEN 系统 SHALL 将其识别为 `/help` 命令（忽略前后空格）

### 需求 3：帮助信息返回

**用户故事：** 作为一名用户，我希望在输入 `/help` 命令后看到清晰的帮助信息，以便了解系统支持哪些命令。

#### 验收标准

1. WHEN `handle_cmd` 方法处理 `/help` 命令 THEN 系统 SHALL 返回包含可用命令列表的响应消息
2. WHEN 返回帮助信息 THEN 系统 SHALL 使用与 Python 版本一致的格式和内容：
   - 标题：`🐈 nanobot commands:`
   - 命令列表：`/new — Start a new conversation` 和 `/help — Show available commands`
3. WHEN 返回帮助信息 THEN 系统 SHALL 在 `OutboundMessage` 中正确设置 `channel` 和 `chat_id` 字段，确保消息路由到正确的目标

### 需求 4：命令处理流程集成

**用户故事：** 作为一名开发者，我希望命令处理逻辑能够无缝集成到现有的消息处理流程中，以便保持代码结构的一致性和可维护性。

#### 验收标准

1. WHEN `process_message` 方法接收到消息 THEN 系统 SHALL 首先检查消息是否以 `/` 开头来判断是否为命令
2. IF 消息被识别为命令 THEN 系统 SHALL 调用 `handle_cmd` 方法并直接返回命令响应，跳过后续的 LLM 调用和工具执行流程
3. WHEN 处理 `/help` 命令 THEN 系统 SHALL NOT 创建或修改会话状态（不保存历史消息）
4. WHEN 处理 `/help` 命令 THEN 系统 SHALL NOT 触发记忆整合任务

### 需求 5：代码可扩展性

**用户故事：** 作为一名开发者，我希望命令处理逻辑具有良好的可扩展性，以便未来能够方便地添加更多命令（如 `/new`）。

#### 验收标准

1. WHEN 添加新命令 THEN 系统 SHALL 只需在 `handle_cmd` 方法中添加新的分支，而不需要修改核心消息处理流程
2. WHEN 实现 `handle_cmd` 方法 THEN 系统 SHALL 采用 match 或类似的结构化方式处理不同命令

## 技术约束

1. 必须与 Python 版本的命令行为保持一致
2. 必须遵循 Rust 版本现有的代码风格和架构模式
3. 帮助信息的内容和格式必须与 Python 版本完全一致

## 成功标准

1. 用户输入 `/help` 后能够收到正确的帮助信息
2. 帮助信息内容与 Python 版本一致
3. 代码结构清晰，易于扩展其他命令
4. 不影响现有的消息处理流程
