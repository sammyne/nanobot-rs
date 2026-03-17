# 需求文档

## 引言

本需求文档描述了对 `AgentLoop::try_consolidate` 方法的重构工作。当前实现存在职责不清的问题：方法内部既执行记忆整合，又修改 session 状态并持久化会话。这违反了单一职责原则，增加了测试难度，并导致调用方无法感知状态变更。

重构的目标是将 `try_consolidate` 改造为纯计算方法，只负责执行记忆整合并返回结果，由调用方负责状态更新和持久化。

## 需求

### 需求 1：方法签名重构

**用户故事：** 作为开发者，我希望 `try_consolidate` 方法具有清晰的输入输出契约，以便于理解、测试和维护。

#### 验收标准

1. WHEN 调用 `try_consolidate` 方法 THEN 系统 SHALL 接受不可变的 session 引用（`&Session`）作为参数
2. WHEN 记忆整合成功且产生新的整合点 THEN 系统 SHALL 返回 `Ok(Some(new_last_consolidated))`
3. WHEN 记忆整合成功但无需更新 THEN 系统 SHALL 返回 `Ok(None)`
4. WHEN 记忆整合过程发生错误 THEN 系统 SHALL 返回 `Err(e)` 并记录错误日志

### 需求 2：移除副作用

**用户故事：** 作为开发者，我希望 `try_consolidate` 方法不产生副作用，以便于进行单元测试和推理代码行为。

#### 验收标准

1. WHEN 执行 `try_consolidate` 方法 THEN 系统 SHALL NOT 修改传入的 session 对象
2. WHEN 执行 `try_consolidate` 方法 THEN 系统 SHALL NOT 调用 `sessions.save()` 进行持久化
3. IF 记忆整合需要更新状态 THEN 系统 SHALL 仅返回新的 `last_consolidated` 值，由调用方决定如何处理

### 需求 3：调用方适配

**用户故事：** 作为开发者，我希望调用方代码正确处理 `try_consolidate` 的返回值，以便正确更新 session 状态并持久化。

#### 验收标准

1. WHEN `process_message` 方法调用 `try_consolidate` THEN 系统 SHALL 检查返回的 `Option<usize>` 值
2. IF 返回值为 `Some(new_value)` THEN 系统 SHALL 更新 `session.last_consolidated = new_value`
3. IF session 状态被更新 THEN 系统 SHALL 调用 `self.sessions.save(&session)` 进行持久化
4. IF 持久化失败 THEN 系统 SHALL 记录错误日志但不中断消息处理流程

### 需求 4：错误处理

**用户故事：** 作为开发者，我希望错误处理逻辑清晰且一致，以便系统在异常情况下仍能稳定运行。

#### 验收标准

1. WHEN `try_consolidate` 内部调用 `memory().try_consolidate()` 失败 THEN 系统 SHALL 返回错误并记录日志
2. WHEN 调用方收到错误 THEN 系统 SHALL 记录错误日志但不影响消息处理的主流程
3. WHEN 持久化失败 THEN 系统 SHALL 记录错误日志但不抛出异常

## 技术约束

- 保持与现有 `nanobot_memory::MemoryStore::try_consolidate` 接口的兼容性
- 遵循 Rust 的错误处理最佳实践（使用 `anyhow::Result`）
- 保持日志记录的完整性（使用 `tracing` 宏）
