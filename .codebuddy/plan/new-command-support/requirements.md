# 需求文档

## 引言

本文档描述了在 Rust 版本的 `AgentLoop` 中实现 `/new` 命令的需求。`/new` 命令允许用户在当前会话中清除所有对话消息并开始一个新的对话，同时保留之前对话的重要信息到长期记忆（MEMORY.md）和历史日志（HISTORY.md）中。

该功能对于：
- 当用户希望重置对话上下文时非常有用
- 确保不会丢失之前的重要对话信息
- 与 Python 版本的 nanobot 保持一致的用户体验

## 需求

### 需求 1：`/new` 命令识别

**用户故事：** 作为一名 nanobot 用户，我希望可以通过发送 `/new` 命令来启动新的对话，以便清除当前会话的历史消息并开始一个新的对话。

#### 验收标准

1. WHEN 用户发送消息内容为 `/new`（大小写不敏感） THEN 系统 SHALL 将其识别为命令并触发新会话流程
2. WHEN 消息内容为 `/new` THEN 系统 SHALL 不将消息作为普通对话内容传递给 LLM

### 需求 2：消息归档到记忆存储

**用户故事：** 作为一名 nanobot 用户，我希望在清除会话之前，当前对话的重要信息能够被保存到长期记忆中，以便未来对话时可以参考这些信息。

#### 验收标准

1. WHEN 收到 `/new` 命令 THEN 系统 SHALL 获取当前会话从 `last_consolidated` 位置到末尾的所有未整合消息
2. WHEN 存在未整合消息 THEN 系统 SHALL 使用 `MemoryStore.consolidate()` 方法将这些消息归档（设置 `archive_all=true`）
3. WHEN 记忆归档成功 THEN 系统 SHALL 继续执行会话清除流程
4. WHEN 记忆归档失败 THEN 系统 SHALL 返回错误消息 "Memory archival failed, session not cleared. Please try again."

### 需求 3：并发控制

**用户故事：** 作为一名 nanobot 系统设计者，我希望系统能够防止同一会话的并发归档操作，以避免数据竞争和状态不一致的问题。

#### 验收标准

1. WHEN 收到 `/new` 命令 THEN 系统 SHALL 通过 `consolidating.lock()` 获取互斥锁
2. WHEN 获取到互斥锁 THEN 系统 SHALL 在执行归档前检查会话是否已在 `consolidating` 集合中
3. WHEN 会话不在 `consolidating` 集合中 THEN 系统 SHALL 添加会话标记并执行归档操作
4. WHEN 归档完成或失败 THEN 系统 SHALL 从 `consolidating` 集合中移除会话标记
5. WHEN 发生异常 THEN 系统 SHALL 确保会话标记已被移除（即使在异常情况下）

### 需求 4：会话清除

**用户故事：** 作为一名 nanobot 用户，我希望在消息成功归档后，当前会话被完全清除，以便我可以从零开始新的对话。

#### 验收标准

1. WHEN 记忆归档成功 THEN 系统 SHALL 调用 `session.clear()` 方法清除所有消息并重置 `last_consolidated` 为 0
2. WHEN 会话被清除 THEN 系统 SHALL 将清除后的会话保存到存储中
3. WHEN 会话被清除 THEN 系统 SHALL 调用 `SessionManager.invalidate()` 失效会话缓存（如果有）
4. WHEN 会话清除完成 THEN 系统 SHALL 返回成功消息 "New session started."

### 需求 5：错误处理

**用户故事：** 作为一名 nanobot 用户，我希望在 `/new` 命令执行过程中出现错误时，能够收到明确的错误提示，并且不会意外地丢失会话数据。

#### 验收标准

1. WHEN 执行 `/new` 命令过程中发生任何异常 THEN 系统 SHALL 捕获该异常并记录错误日志
2. WHEN 发生异常 THEN 系统 SHALL 确保执行清理操作（从 `consolidating` 集合移除会话，清理整合锁）
3. WHEN 发生异常 THEN 系统 SHALL 返回错误消息 "Memory archival failed, session not cleared. Please try again."
4. WHEN 发生异常 THEN 系统 SHALL 不清除会话消息（确保不会丢失数据）

### 需求 6：集成到命令处理流程

**用户故事：** 作为一名 nanobot 开发者，我希望 `/new` 命令能够集成到现有的命令处理流程中，与其他命令（如 `/help`）使用相同的处理机制。

#### 验收标准

1. WHEN 收到 `/` 开头的消息 THEN 系统 SHALL 在 `try_handle_cmd()` 方法中检查命令是否为 `new`
2. WHEN 命令为 `new` THEN 系统 SHALL 执行上述新会话流程并返回相应的 `OutboundMessage`
3. WHEN `/new` 命令执行完成 THEN 系统 SHALL 不继续执行后续的普通消息处理流程
