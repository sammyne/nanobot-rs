# 需求文档

## 引言

ContextBuilder 是 nanobot 的核心组件之一，负责为 LLM 构建上下文（系统提示词 + 消息列表）。该组件将 Python 版本移植到 Rust，与现有的 memory、session、provider 模块集成，实现高效的上下文构建能力。

ContextBuilder 的主要职责包括：
- 加载 workspace 下的 bootstrap 文件（AGENTS.md、SOUL.md 等）
- 组装系统提示词（身份信息 + bootstrap 文件 + 记忆上下文 + 技能摘要）
- 构建完整的消息列表供 LLM 调用
- 注入运行时上下文（当前时间、渠道信息等）
- 支持多媒体消息（图片的 base64 编码）

## 需求

### 需求 1：核心结构与初始化

**用户故事：** 作为开发者，我希望 ContextBuilder 能够初始化并持有必要的依赖，以便后续构建上下文时能够访问 memory 等组件。

#### 验收标准

1. WHEN 创建 ContextBuilder 实例 THEN 系统 SHALL 接受 workspace 路径作为参数
2. WHEN 初始化 ContextBuilder THEN 系统 SHALL 创建 MemoryStore 实例用于访问记忆
3. IF workspace 路径不存在 THEN 系统 SHALL 返回错误而非 panic

### 需求 3：系统提示词构建

**用户故事：** 作为开发者，我希望 ContextBuilder 能够组装完整的系统提示词，以便 LLM 能够理解其身份、能力和当前上下文。

#### 验收标准

1. WHEN 构建系统提示词 THEN 系统 SHALL 首先包含核心身份部分（nanobot 介绍、运行时信息、工作空间路径）
2. WHEN 构建系统提示词 THEN 系统 SHALL 包含已加载的 bootstrap 文件内容
3. WHEN 记忆存储中有内容 THEN 系统 SHALL 在系统提示词中包含 `# Memory` 章节
4. WHEN 构建系统提示词 THEN 系统 SHALL 包含技能摘要章节，指导 LLM 如何使用技能
5. WHEN 各部分都构建完成 THEN 系统 SHALL 使用 `---` 分隔符连接各部分

### 需求 4：核心身份信息生成

**用户故事：** 作为开发者，我希望系统提示词包含运行时身份信息，以便 LLM 了解其运行环境和工作空间位置。

#### 验收标准

1. WHEN 生成核心身份 THEN 系统 SHALL 包含 nanobot 的基本介绍
2. WHEN 生成核心身份 THEN 系统 SHALL 包含运行时信息（操作系统、架构）
3. WHEN 生成核心身份 THEN 系统 SHALL 包含工作空间的绝对路径
4. WHEN 生成核心身份 THEN 系统 SHALL 包含记忆文件路径（MEMORY.md、HISTORY.md）
5. WHEN 生成核心身份 THEN 系统 SHALL 包含工具调用指南

### 需求 5：运行时上下文注入

**用户故事：** 作为开发者，我希望在用户消息末尾注入运行时上下文，以便 LLM 能够感知当前时间和渠道信息。

#### 验收标准

1. WHEN 注入运行时上下文 THEN 系统 SHALL 在消息末尾添加当前时间（格式：YYYY-MM-DD HH:MM (Weekday)）
2. WHEN 存在渠道信息 THEN 系统 SHALL 添加 `Channel: {channel}` 信息
3. WHEN 存在聊天 ID THEN 系统 SHALL 添加 `Chat ID: {chat_id}` 信息
4. WHEN 消息内容为字符串 THEN 系统 SHALL 返回字符串格式
5. WHEN 消息内容为多媒体数组 THEN 系统 SHALL 在数组末尾追加运行时上下文

### 需求 6：消息列表构建

**用户故事：** 作为开发者，我希望 ContextBuilder 能够构建完整的消息列表，以便直接传递给 LLM Provider 进行调用。

#### 验收标准

1. WHEN 构建消息列表 THEN 系统 SHALL 首先添加系统提示词作为 system 消息
2. WHEN 构建消息列表 THEN 系统 SHALL 追加历史消息
3. WHEN 构建消息列表 THEN 系统 SHALL 追加当前用户消息（带运行时上下文）
4. IF 存在媒体文件 THEN 系统 SHALL 将图片编码为 base64 格式
5. WHEN 消息列表构建完成 THEN 系统 SHALL 返回 `Vec<Message>` 类型

### 需求 7：媒体文件处理

**用户故事：** 作为开发者，我希望能够将图片文件编码为 base64 格式并附加到用户消息，以便支持多模态 LLM 调用。

#### 验收标准

1. WHEN 处理媒体文件 THEN 系统 SHALL 检测文件的 MIME 类型
2. IF 文件不存在或非图片类型 THEN 系统 SHALL 跳过该文件
3. WHEN 成功读取图片 THEN 系统 SHALL 编码为 base64 并构建 `data:{mime};base64,{data}` 格式
4. WHEN 存在多个图片 THEN 系统 SHALL 将所有图片放在文本内容之前

### 需求 8：消息追加辅助方法

**用户故事：** 作为开发者，我希望能够方便地向消息列表追加工具结果和助手消息，以便在 agent loop 中管理对话状态。

#### 验收标准

1. WHEN 添加工具结果 THEN 系统 SHALL 创建 tool 角色消息，包含 tool_call_id 和内容
2. WHEN 添加助手消息 THEN 系统 SHALL 支持 content 和 tool_calls 参数
3. WHEN 添加助手消息 AND 包含 reasoning_content THEN 系统 SHALL 保留推理内容字段

### 需求 9：错误处理

**用户故事：** 作为开发者，我希望 ContextBuilder 使用库项目的错误处理规范（thiserror），以便调用者能够精确处理错误。

#### 验收标准

1. WHEN 定义错误类型 THEN 系统 SHALL 使用 `thiserror` 库
2. WHEN 文件读取失败 THEN 系统 SHALL 返回 `Io` 错误变体
3. WHEN 路径无效 THEN 系统 SHALL 返回 `InvalidPath` 错误变体
4. WHEN MIME 类型检测失败 THEN 系统 SHALL 返回 `MediaType` 错误变体
5. 每个错误变体都不带 `Error` 前缀或后缀

### 需求 10：模块结构与测试

**用户故事：** 作为开发者，我希望 ContextBuilder 遵循项目的模块结构和测试规范，以便代码风格一致且易于维护。

#### 验收标准

1. WHEN 创建模块 THEN 系统 SHALL 在 `crates/context/` 目录下创建新 crate
2. WHEN 组织代码 THEN 系统 SHALL 将源代码放在 `mod.rs`，测试代码放在同级 `tests.rs`
3. WHEN 编写测试 THEN 系统 SHALL 不使用 `test_` 前缀命名测试函数
4. WHEN 定义公共 API THEN 系统 SHALL 在 `lib.rs` 中导出核心类型
