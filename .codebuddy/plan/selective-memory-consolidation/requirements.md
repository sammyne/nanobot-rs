# 需求文档

## 引言

本功能旨在优化 `AgentLoop::process_message` 中的记忆整合触发机制。当前实现每次消息处理后都会调用 `try_consolidate`，虽然内部有条件检查，但缺乏对"进行中整合任务"的追踪。本需求将实现更精确的选择性记忆整合，确保：

1. 仅在消息数量达到阈值时触发整合
2. 避免在已有整合任务进行中时重复触发

## 需求

### 需求 1：基于消息窗口的整合触发条件

**用户故事：** 作为系统运维人员，我希望记忆整合仅在消息数量达到配置阈值时触发，以便优化系统资源使用并避免不必要的 LLM 调用。

#### 验收标准

1. WHEN `len(session.messages) - session.last_consolidated >= memory_window` THEN 系统 SHALL 触发记忆整合
2. WHEN `len(session.messages) - session.last_consolidated < memory_window` THEN 系统 SHALL 跳过记忆整合检查

### 需求 2：整合条件组合判断

**用户故事：** 作为系统架构师，我希望整合触发条件清晰且可维护，以便后续扩展和调试。

#### 验收标准

1. WHEN 检查是否应执行整合 THEN 系统 SHALL 满足需求 1 的消息窗口条件，且会话未处于整合中状态
2. IF 任一条件不满足 THEN 系统 SHALL 跳过整合并继续正常消息处理流程
3. WHEN 整合条件满足 THEN 系统 SHALL 在当前消息处理完成后、会话持久化前执行整合

### 需求 3：AgentLoop 结构扩展

**用户故事：** 作为系统开发者，我希望 AgentLoop 结构能支持整合状态追踪，以便实现需求 2 的状态管理。

#### 验收标准

1. WHEN 定义 AgentLoop 结构 THEN 系统 SHALL 包含 `consolidating: HashSet<SessionId>` 字段（或等效的会话状态集合）
2. WHEN 创建 AgentLoop 实例 THEN 系统 SHALL 将整合状态集合初始化为空
3. 由于整合状态仅用于运行时控制，系统 SHALL NOT 将其持久化到会话存储
4. WHEN 检查会话的整合状态 THEN 系统 SHALL 检查会话 ID 是否存在于集合中，若存在则表示正在整合

## 技术说明

### 现有代码分析

- 当前 `should_consolidate` 方法使用 `keep_count = memory_window / 2` 作为阈值
- 新需求使用完整的 `memory_window` 作为阈值，语义更清晰
- 当前 `try_consolidate` 是同步执行的，不存在真正的并发问题，但状态追踪有助于未来异步化

### 线程安全评估

**结论：需要加锁**

**并发访问场景分析：**

1. **Gateway 模式下的多入口访问**：
   - `run` 方法在独立的 tokio task 中运行，通过 mpsc 通道处理消息
   - `process_direct` 方法被 cron 回调调用（见 `setup_cron_callback`）
   - 这两个入口点可能同时访问 `consolidating` 状态

2. **当前 AgentLoop 的线程安全性**：
   - `sessions: Arc<SessionManager>` - SessionManager 内部使用 `RwLock` 保护缓存
   - `tool_registry: ToolRegistry` - 使用普通 HashMap，但仅在初始化时修改
   - `context: ContextBuilder` - 需要检查其线程安全性

3. **内部可变性需求**：
   - `run` 和 `process_direct` 方法都使用 `&self`（不可变引用）
   - 修改 `consolidating` 状态需要内部可变性模式

**推荐方案：**

使用 `Mutex<HashSet<SessionId>>` 包装 `consolidating` 字段：

```rust
pub struct AgentLoop<P: Provider + 'static> {
    // ... existing fields ...
    consolidating: Mutex<HashSet<String>>,  // 使用 session_key 作为标识
}
```

**选择 Mutex 而非 RwLock 的理由：**
- 整合状态检查和修改都是短时间操作
- 不存在大量读操作的场景
- Mutex 实现更简单，性能开销更低

### 实现建议

1. 在 `AgentLoop` 结构中添加 `consolidating: Mutex<HashSet<String>>` 字段
2. 修改 `process_message` 中的整合逻辑，检查消息窗口条件和整合状态后再调用 `try_consolidate`
3. 在整合开始前通过 `consolidating.lock().insert(session_key)` 标记状态
4. 完成后通过 `consolidating.lock().remove(&session_key)` 清除状态
