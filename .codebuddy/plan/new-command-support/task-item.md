# 实施计划

- [ ] 1. 在命令处理方法中添加 `/new` 命令识别
  - 在 `try_handle_cmd()` 方法中添加 `new` 命令的匹配逻辑（大小写不敏感）
  - 创建占位函数 `handle_new_command()` 返回待实现的 `OutboundMessage`
  - 确保命令识别后不传递给 LLM 处理
  - _需求：1.1、1.2、6.1_

- [ ] 2. 实现 `/new` 命令的核心处理函数
  - 创建 `async fn handle_new_command()` 方法，接收必要的参数（session, msg）
  - 定义函数签名返回 `Result<OutboundMessage, AgentError>`
  - 添加基本函数结构和日志记录
  - 注意：该函数将被 `tokio::spawn` 异步执行，以确保锁能够正确释放
  - _需求：6.2、6.3_

- [ ] 3. 实现消息归档逻辑
  - 在核心处理函数中获取从 `last_consolidated` 到末尾的未整合消息切片
  - 调用 `MemoryStore.consolidate()` 方法，传入临时会话和 `archive_all=true` 参数
  - 实现成功/失败的消息返回逻辑
  - _需求：2.1、2.2、2.3、2.4_

- [ ] 4. 实现并发控制机制
  - 使用 `consolidating.lock()` 获取互斥锁
  - 检查会话是否已在 `consolidating` 集合中，避免重复归档
  - 添加会话标记到 `consolidating` 集合
  - 在归档完成后（无论成功或失败）从 `consolidating` 集合中移除会话标记
  - 使用统一错误处理确保异常情况下也能清理标记
  - _需求：3.1、3.2、3.3、3.4、3.5_

- [ ] 5. 实现会话清除和存储更新
  - 调用 `session.clear()` 清除所有消息并重置 `last_consolidated` 为 0
  - 调用 `SessionManager.save()` 保存清除后的会话
  - 调用 `SessionManager.invalidate()` 失效会话缓存
  - 返回成功消息 "New session started."
  - _需求：4.1、4.2、4.3、4.4_

- [ ] 7. 实现错误处理机制
  - 使用 `match` 或 `?` 操作符捕获归档过程中的异常
  - 记录详细的错误日志（包含会话 key 和异常信息）
  - 确保异常情况下会话标记已被清理
  - 返回错误消息 "Memory archival failed, session not cleared. Please try again."
  - 确保异常发生时不清除会话消息
  - _需求：5.1、5.2、5.3、5.4_

- [ ] 8. 集成到命令处理流程
  - 在 `try_handle_cmd()` 方法中识别 `/new` 命令后，使用 `tokio::spawn` 异步执行 `handle_new_command()`
  - 使用 `JoinHandle` 等待异步任务完成，获取执行结果
  - 根据执行结果返回适当的响应消息给用户
  - 确保命令识别后跳过后续的普通消息处理
  - 将必要的数据（如 `session_key`、`MemoryStore` 的引用等）直接 move 到 spawn 的任务闭包中
  - _需求：6.2、6.3_

- [ ] 9. 添加单元测试
  - 为 `/new` 命令识别编写测试用例
  - 为消息归档成功和失败场景编写测试
  - 为并发控制编写并发测试用例
  - 为错误处理编写测试用例
  - _需求：所有需求_

- [ ] 10. 添加集成测试和文档
  - 编写端到端的集成测试，验证完整的 `/new` 命令流程
  - 在代码中添加详细的注释说明实现逻辑
  - 更新 README 或相关文档说明 `/new` 命令的使用方法
  - _需求：所有需求_
