# 需求文档

## 引言

本项目旨在为 nanobot-rs 添加会话管理功能，参照 Python 版本的实现（`_nanobot/nanobot/session/manager.py`）。当前 Rust 版本仅使用内存中的 `HashMap` 存储会话，缺乏持久化能力和完整的会话生命周期管理。本功能将实现基于 JSONL 文件格式的会话持久化、内存缓存、以及会话列表查询等核心能力。

## 需求

### 需求 1

**用户故事：** 作为一名 nanobot 用户，我希望我的对话历史能够持久化保存，以便在重启应用后能够继续之前的对话。

#### 验收标准

1. WHEN 创建新会话 THEN 系统 SHALL 在工作目录的 `sessions/` 子目录下创建对应的 `.jsonl` 文件
2. WHEN 会话内容更新 THEN 系统 SHALL 将会话消息以 JSONL 格式（每行一个 JSON 对象）持久化到文件
3. WHEN 应用重启后访问已有会话 THEN 系统 SHALL 从磁盘加载历史消息并恢复会话状态
4. IF 会话文件损坏或格式错误 THEN 系统 SHALL 记录警告日志并返回空会话，而非崩溃

### 需求 2

**用户故事：** 作为一名 nanobot 开发者，我希望会话管理器支持内存缓存，以便减少频繁的磁盘 I/O 操作。

#### 验收标准

1. WHEN 首次访问某个会话 THEN 系统 SHALL 从磁盘加载会话并存入内存缓存
2. WHEN 再次访问同一会话 THEN 系统 SHALL 直接从缓存返回，不重复读取磁盘
3. WHEN 调用 `invalidate` 方法 THEN 系统 SHALL 从缓存中移除指定会话
4. WHEN 保存会话 THEN 系统 SHALL 同时更新缓存和磁盘文件

### 需求 3

**用户故事：** 作为一名 nanobot 用户，我希望每个会话能够记录创建时间、更新时间等元数据，以便追踪会话的生命周期。

#### 验收标准

1. WHEN 创建新会话 THEN 系统 SHALL 自动设置 `created_at` 和 `updated_at` 为当前时间
2. WHEN 会话内容变更（添加消息、清除等） THEN 系统 SHALL 更新 `updated_at` 时间戳
3. WHEN 持久化会话 THEN 系统 SHALL 将元数据作为 JSONL 文件的首行存储（标记 `_type: "metadata"`）
4. WHEN 加载会话 THEN 系统 SHALL 从元数据行解析 `created_at`、`updated_at`、`metadata`、`last_consolidated` 等字段

### 需求 4

**用户故事：** 作为一名 nanobot 用户，我希望会话支持历史消息窗口限制，以便控制内存使用和 LLM 上下文长度。

#### 验收标准

1. WHEN 获取会话历史 THEN 系统 SHALL 支持指定 `max_messages` 参数限制返回的消息数量
2. WHEN 返回的历史被截断 THEN 系统 SHALL 确保第一条返回的消息是 `user` 角色，避免孤立的 `tool_result` 块
3. WHEN 返回历史消息 THEN 系统 SHALL 只包含 `role`、`content`、`tool_calls`、`tool_call_id`、`name` 等 LLM 所需字段

### 需求 5

**用户故事：** 作为一名 nanobot 用户，我希望能够清除当前会话并开始新对话，同时保留之前的对话历史用于记忆归档。

#### 验收标准

1. WHEN 调用会话的 `clear` 方法 THEN 系统 SHALL 清空消息列表并重置 `last_consolidated` 为 0
2. WHEN 清除会话 THEN 系统 SHALL 更新 `updated_at` 时间戳

### 需求 6

**用户故事：** 作为一名 nanobot 开发者，我希望能够列出所有会话及其基本信息，以便实现会话管理界面或调试。

#### 验收标准

1. WHEN 调用 `list_sessions` 方法 THEN 系统 SHALL 返回所有会话的基本信息列表
2. WHEN 返回会话列表 THEN 系统 SHALL 包含 `key`、`created_at`、`updated_at`、`path` 字段
3. WHEN 返回会话列表 THEN 系统 SHALL 按 `updated_at` 降序排列

### 需求 7

**用户故事：** 作为一名 nanobot 开发者，我希望会话管理模块能够与现有 AgentLoop 无缝集成，以便保持代码一致性。

#### 验收标准

1. WHEN 实现 SessionManager THEN 系统 SHALL 提供 `get_or_create`、`save`、`invalidate` 等与 Python 版本兼容的方法签名
2. WHEN 替换 AgentLoop 中的会话存储 THEN 系统 SHALL 使用新的 `SessionManager` 替代 `HashMap<String, Vec<Message>>`
3. WHEN AgentLoop 处理消息 THEN 系统 SHALL 通过 SessionManager 管理会话生命周期
