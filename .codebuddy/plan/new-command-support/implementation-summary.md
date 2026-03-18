# /new 命令实现总结

## 实现概述

成功实现了 `/new` 命令功能，允许用户启动新会话，将未整合的消息归档到长期记忆，然后清除当前会话。

## 已完成的任务

### 1. 命令识别和处理 ✅
- 在 `try_handle_cmd()` 方法中添加了 `new` 命令匹配逻辑
- 支持大小写不敏感的命令识别（`/new`、`/NEW`、`/New`）
- 忽略前后空格
- 命令识别后不会传递给 LLM 处理

### 2. 核心处理函数 ✅
- 实现了 `async fn handle_new_cmd()` 方法
- 函数签名：`async fn handle_new_cmd(&self, channel: String, chat_id: String) -> Result<String, String>`
- 添加了详细的文档注释和日志记录

### 3. 消息归档逻辑 ✅
- 调用 `MemoryStore.consolidate()` 方法
- 使用 `archive_all=true` 参数强制归档所有未整合消息
- 实现了成功/失败的消息返回逻辑

### 4. 并发控制机制 ✅
- 使用 `consolidating.lock()` 获取互斥锁
- 检查会话是否已在 `consolidating` 集合中
- 添加会话标记到 `consolidating` 集合
- 在归档完成后（无论成功或失败）从 `consolidating` 集合中移除会话标记
- 使用统一的错误处理确保异常情况下也能清理标记

### 5. 会话清除和存储更新 ✅
- 调用 `session.clear()` 清除所有消息并重置 `last_consolidated` 为 0
- 调用 `SessionManager.save()` 保存清除后的会话
- 调用 `SessionManager.invalidate()` 失效会话缓存
- 返回成功消息 "New session started."

### 6. 错误处理机制 ✅
- 使用 `match` 操作符捕获归档过程中的异常
- 记录详细的错误日志（包含会话 key 和异常信息）
- 确保异常情况下会话标记已被清理
- 返回适当的错误消息
- 确保异常发生时不清除会话消息

### 7. 集成到命令处理流程 ✅
- 在 `try_handle_cmd()` 方法中识别 `/new` 命令后，使用 `tokio::spawn` 异步执行 `handle_new_cmd()`
- 使用 `JoinHandle` 等待异步任务完成，获取执行结果
- 根据执行结果返回适当的响应消息给用户
- 确保命令识别后跳过后续的普通消息处理
- 将必要的数据（`sessions`、`memory`、`provider`、`consolidating`）直接 move 到 spawn 的任务闭包中

### 8. 单元测试 ✅
添加了以下测试用例：
- `try_handle_cmd_recognizes_new_command`: 验证 `/new` 命令的识别和处理（大小写不敏感）
- `new_command_clears_session_history`: 验证 `/new` 命令清除会话历史
- `new_command_handles_concurrent_requests`: 验证 `/new` 命令处理并发请求
- `new_command_returns_error_when_consolidating`: 验证 `/new` 命令在整合进行时返回错误

### 9. 文档注释 ✅
- 为 `handle_new_cmd()` 方法添加了详细的文档注释
- 包含方法描述、参数说明、返回值说明和使用示例
- 在代码关键位置添加了注释说明实现逻辑

## 技术亮点

1. **异步处理**: 使用 `tokio::spawn` 异步执行 `/new` 命令，避免阻塞主循环
2. **并发安全**: 使用 `Arc<Mutex<HashSet<String>>>` 确保并发控制的安全性
3. **所有权管理**: 将必要的数据直接 move 到 spawn 任务闭包中，避免 clone
4. **错误处理**: 完善的错误处理机制，确保资源正确释放
5. **线程安全**: 在并发场景下正确处理锁的状态

## 文件修改

- `crates/agent/src/loop/mod.rs`: 添加 `/new` 命令实现
- `crates/agent/src/loop/tests.rs`: 添加 `/new` 命令测试用例

## 使用方法

用户可以在聊天中输入 `/new` 命令来启动新会话：

```
/new
```

系统会：
1. 将当前会话的未整合消息归档到长期记忆
2. 清除当前会话的所有消息
3. 返回 "New session started." 消息

## 错误处理

如果归档失败，系统会：
1. 返回错误消息 "Memory archival failed, session not cleared. Please try again."
2. 不清除会话消息
3. 清理 consolidating 状态标记

如果会话正在进行整合，系统会：
1. 返回错误消息 "Session is already being consolidated. Please try again later."
2. 不执行归档操作
