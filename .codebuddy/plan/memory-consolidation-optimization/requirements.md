# 需求文档

## 引言
本需求旨在优化 `AgentLoop::process_message` 方法中的记忆整合逻辑。当前的实现在消息处理完成后（save_turn 之后）执行记忆整合，这可能导致不必要的延迟和潜在的并发问题。通过将记忆整合逻辑移动到 build_messages 之前启动异步执行，并在 re_act 之后等待整合完成再执行 save_turn，可以更好地利用异步并行能力，提升消息处理的整体效率。

## 需求

### 需求 1：在消息构建前启动异步记忆整合

**用户故事：** 作为一名系统优化者，我希望在调用 build_messages 之前启动记忆整合的异步任务，以便记忆整合与后续的 LLM 调用能够并行执行，提升整体响应速度。

#### 验收标准

1. WHEN `process_message` 获取会话历史后
   THEN 系统 SHALL 在调用 `build_messages` 之前启动异步记忆整合任务

2. WHEN 系统检测到需要整合（消息数量达到阈值且无进行中的整合任务）
   THEN 系统 SHALL 使用 `tokio::spawn` 启动记忆整合任务，立即返回一个 `JoinHandle`

3. WHEN 系统检测到不需要整合（消息数量未达到阈值或已有整合任务在进行）
   THEN 系统 SHALL 设置整合句柄为 `None`，继续后续流程

### 需求 2：在 ReAct 循环后等待整合完成

**用户故事：** 作为一名系统优化者，我希望在 re_act 完成后等待记忆整合任务完成，以便确保在保存消息前整合已经结束，保证数据一致性。

#### 验收标准

1. WHEN `re_act` 循环完成并返回结果
   THEN 系统 SHALL 检查是否存在异步整合任务的 `JoinHandle`

2. WHEN 整合任务的 `JoinHandle` 存在
   THEN 系统 SHALL 使用 `await` 等待整合任务完成并获取结果

3. WHEN 整合任务成功完成且返回新的 `last_consolidated` 值
   THEN 系统 SHALL 更新会话的 `last_consolidated` 字段

4. WHEN 整合任务失败或返回 `None`
   THEN 系统 SHALL 记录错误日志但不中断消息处理流程

5. WHEN 整合任务的 `JoinHandle` 不存在（满足以下任一条件）：
   - 消息数量未达到整合阈值（即 `history.len() - last_consolidated < threshold`）
   - 已有整合任务正在进行中（通过 `consolidating` Mutex 检查发现会话 ID 已在集合中）
   - 会话历史为空或不足以进行整合
   THEN 系统 SHALL 跳过等待步骤，直接执行后续操作

### 需求 3：在整合完成后执行 save_turn

**用户故事：** 作为一名数据一致性保障者，我希望在记忆整合完成后再保存本回合消息，以便确保保存的消息状态与整合后的状态一致。

#### 验收标准

1. WHEN 记忆整合任务完成（无论是成功还是失败）
   THEN 系统 SHALL 调用 `session.save_turn` 保存本回合消息

2. WHEN 保存消息成功
   THEN 系统 SHALL 继续执行会话持久化操作

3. WHEN 保存消息失败
   THEN 系统 SHALL 记录错误日志但不中断处理流程

### 需求 4：维护整合状态管理

**用户故事：** 作为一名并发控制者，我希望正确管理记忆整合状态，防止同一会话同时进行多个整合任务。

#### 验收标准

1. WHEN 启动异步整合任务前
   THEN 系统 SHALL 使用 `consolidating` Mutex 检查并标记会话为正在整合状态

2. WHEN 异步整合任务完成（无论成功或失败）
   THEN 系统 SHALL 从 `consolidating` HashSet 中移除会话标识

3. WHEN 整合任务在异步上下文中执行
   THEN 系统 SHALL 确保所有必要的资源（如 provider、context）在任务中可用

### 需求 5：错误处理和日志记录

**用户故事：** 作为一名系统运维者，我希望能够清楚地追踪记忆整合的状态和错误，以便于调试和问题定位。

#### 验收标准

1. WHEN 异步整合任务启动
   THEN 系统 SHALL 记录日志显示整合任务已启动

2. WHEN 等待整合任务完成
   THEN 系统 SHALL 记录日志显示正在等待整合完成

3. WHEN 整合任务成功完成
   THEN 系统 SHALL 记录日志显示整合完成及新的 last_consolidated 值

4. WHEN 整合任务失败
   THEN 系统 SHALL 记录详细的错误信息

5. WHEN 整合任务发生错误
   THEN 系统 SHALL 确保 consolidating 状态被清除，防止死锁
