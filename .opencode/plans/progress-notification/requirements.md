# 需求

## 目标与背景

为 nanobot-rs 项目实现请求处理进度通知功能，使用户能够实时了解 Agent 的思考和工具调用过程，提升用户体验和系统透明度。

该功能已在 Python 版本中实现并验证，现需要将其移植到 Rust 版本，保持功能一致性的同时，遵循 Rust 的最佳实践和项目架构规范。

### 核心价值

1. **实时反馈**：用户能够看到 Agent 的思考过程和正在执行的工具操作
2. **透明度提升**：增强系统的可解释性，让用户了解 Agent 的决策过程
3. **用户体验优化**：避免长时间无响应，提供持续的进度更新
4. **调试便利**：便于开发者和用户定位问题所在

## Python 版本实现分析

### 核心设计

Python 版本在 `AgentLoop` 类中通过 `on_progress` 回调参数实现进度通知：

```python
# loop.py 第 180 行
async def _run_agent_loop(
    self,
    initial_messages: list[dict],
    on_progress: Callable[..., Awaitable[None]] | None = None,
) -> tuple[str | None, list[str], list[dict]]:
```

### 回调签名

```python
async def on_progress(content: str, *, tool_hint: bool = False) -> None
```

参数说明：
- `content: str` - 进度内容（思考内容或工具提示）
- `tool_hint: bool` - 是否为工具提示（默认 False）

### 调用时机

在 `_run_agent_loop` 的 ReAct 循环中（第 200-205 行）：

```python
if response.has_tool_calls:
    if on_progress:
        clean = self._strip_think(response.content)
        if clean:
            await on_progress(clean)
        await on_progress(self._tool_hint(response.tool_calls), tool_hint=True)
```

**触发条件**：当 LLM 响应包含工具调用时

**两次调用**：
1. 发送清理后的思考内容（如果有）
2. 发送工具调用提示（格式如 `tool_name("arg...")`）

### 默认实现

`_bus_progress` 函数（第 424-430 行）：

```python
async def _bus_progress(content: str, *, tool_hint: bool = False) -> None:
    meta = dict(msg.metadata or {})
    meta["_progress"] = True
    meta["_tool_hint"] = tool_hint
    await self.bus.publish_outbound(OutboundMessage(
        channel=msg.channel, chat_id=msg.chat_id, content=content, metadata=meta,
    ))
```

**关键点**：
- 通过消息总线发送进度消息
- 使用 metadata 标记消息类型（`_progress: true`）
- 区分普通进度和工具提示（`_tool_hint: bool`）

### 使用场景

1. **交互式模式**（`run` 方法）：默认使用 `_bus_progress`，通过消息总线发送进度
2. **直接调用模式**（`process_direct` 方法）：支持外部传入自定义回调

```python
# loop.py 第 483-494 行
async def process_direct(
    self,
    content: str,
    session_key: str = "cli:direct",
    channel: str = "cli",
    chat_id: str = "direct",
    on_progress: Callable[[str], Awaitable[None]] | None = None,
) -> str:
```

## Rust 实现方案设计

### 架构适配

Rust 版本采用了不同的架构：
- **Python 版本**：使用 `MessageBus` 进行消息传递
- **Rust 版本**：使用 `mpsc::Sender<OutboundMessage>` 直接发送消息

### 已有基础设施

1. **进度消息支持**（`crates/channels/src/messages/mod.rs` 第 108-118 行）：

```rust
pub fn progress(content: impl Into<String>, is_tool_hint: bool) -> Self {
    let mut metadata = HashMap::new();
    metadata.insert("_progress".to_string(), serde_json::Value::Bool(true));
    if is_tool_hint {
        metadata.insert("_tool_hint".to_string(), serde_json::Value::Bool(true));
    }
    Self { 
        channel: String::new(), 
        chat_id: String::new(), 
        content: content.into(), 
        media: Vec::new(), 
        metadata 
    }
}
```

**优势**：已有完整的进度消息构造逻辑，只需集成到 AgentLoop

2. **消息判断方法**（第 120-128 行）：

```rust
pub fn is_progress(&self) -> bool {
    self.metadata.get("_progress").and_then(|v| v.as_bool()).unwrap_or(false)
}

pub fn is_tool_hint(&self) -> bool {
    self.metadata.get("_tool_hint").and_then(|v| v.as_bool()).unwrap_or(false)
}
```

### 实现策略

**关键设计决策**：`on_progress` 作为**方法参数**传递，而非实例字段。这与 Python 版本保持一致。

采用 **Trait + 方法参数** 的方案：

1. **定义进度通知 Trait**

```rust
use async_trait::async_trait;
use anyhow::Result;

/// 进度追踪器
#[async_trait]
pub trait ProgressTracker: Send + Sync {
    /// 追踪进度
    /// 
    /// # 参数
    /// * `content` - 进度内容
    /// * `is_tool_hint` - 是否为工具提示
    /// 
    /// # 返回值
    /// 成功返回 `Ok(())`，失败返回错误
    async fn track(&self, content: String, is_tool_hint: bool) -> Result<()>;
}
```

2. **默认实现：ChannelProgressTracker**

```rust
/// 通过消息通道发送进度的默认实现
pub struct ChannelProgressTracker {
    tx: mpsc::Sender<OutboundMessage>,
    channel: String,
    chat_id: String,
}

#[async_trait]
impl ProgressTracker for ChannelProgressTracker {
    async fn track(&self, content: String, is_tool_hint: bool) -> Result<()> {
        let mut msg = OutboundMessage::progress(content, is_tool_hint);
        msg.channel = self.channel.clone();
        msg.chat_id = self.chat_id.clone();
        
        self.tx.send(msg).await
            .map_err(|e| anyhow::anyhow!("发送进度消息失败: {}", e))?;
        
        Ok(())
    }
}
```

3. **为闭包直接实现 trait**

```rust
/// 为闭包类型直接实现 ProgressTracker
#[async_trait]
impl<F> ProgressTracker for F
where
    F: Fn(String, bool) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync,
{
    async fn track(&self, content: String, is_tool_hint: bool) -> Result<()> {
        self(content, is_tool_hint).await
    }
}
```

**优势**：
- 无需额外的适配器结构体，闭包可以直接作为 `ProgressTracker` 使用
- 返回 `Result<()>` 便于测试失败场景和错误处理

4. **AgentLoop 结构体（不添加字段）**

```rust
pub struct AgentLoop<P: Provider> {
    provider: P,
    config: AgentDefaults,
    sessions: Arc<SessionManager>,
    tool_registry: ToolRegistry,
    context: ContextBuilder,
    consolidating: Arc<Mutex<HashSet<String>>>,
    // ❌ 不添加 on_progress 字段
}
```

5. **re_act 方法添加参数**

```rust
pub async fn re_act(
    &self,
    messages: Vec<Message>,
    channel: &str,
    chat_id: &str,
    on_progress: Option<Arc<dyn ProgressTracker>>,  // 方法参数
) -> Result<ReActResult> {
    // ... 现有逻辑 ...
    
    if !tool_calls.is_empty() {
        // 发送进度通知
        if let Some(ref tracker) = on_progress {
            // 1. 发送思考内容（如果有）
            if !content.is_empty() {
                if let Err(e) = tracker.track(content.clone(), false).await {
                    error!("发送思考进度失败: {}", e);
                }
            }
            
            // 2. 发送工具提示
            if let Err(e) = tracker.track(hint, true).await {
                error!("发送工具提示失败: {}", e);
            }
        }
        
        // ... 执行工具 ...
    }
}
```

### 方法签名调整

#### 1. `new` 方法（签名不变）

```rust
pub async fn new(
    provider: P,
    config: AgentDefaults,
    cron_service: Option<Arc<CronService>>,
    subagent_manager: Option<Arc<SubagentManager<P>>>,
    tools_config: nanobot_config::ToolsConfig,
) -> Result<Self>
```

#### 2. `re_act` 方法（添加参数）

```rust
pub async fn re_act(
    &self,
    messages: Vec<Message>,
    channel: &str,
    chat_id: &str,
    on_progress: Option<Arc<dyn ProgressTracker>>, // 新增参数
) -> Result<ReActResult>
```

#### 3. `run` 方法（签名不变，内部创建默认回调）

```rust
pub async fn run(
    &self,
    inbound_rx: mpsc::Receiver<InboundMessage>,
    outbound_tx: mpsc::Sender<OutboundMessage>,
) -> Result<()> {
    // 在消息处理内部创建默认进度回调（类似 Python 的 _bus_progress）
    while let Some(msg) = inbound_rx.recv().await {
        let default_tracker = Arc::new(ChannelProgressTracker {
            tx: outbound_tx.clone(),
            channel: msg.channel.clone(),
            chat_id: msg.chat_id.clone(),
        });
        
        self.re_act(messages, &msg.channel, &msg.chat_id, Some(default_tracker)).await?;
    }
}
```

#### 4. `process_direct` 方法（添加参数）

```rust
pub async fn process_direct(
    &self,
    content: &str,
    session_key: &str,
    channel: Option<&str>,
    chat_id: Option<&str>,
    on_progress: Option<Arc<dyn ProgressTracker>>, // 新增参数
) -> Result<String>
```

## 接口定义建议

### 公共 API

```rust
// 在 crates/agent/src/lib.rs 中导出
pub use crate::progress::{ProgressTracker, ChannelProgressTracker};
```

### 使用示例

#### 场景 1：交互式模式（使用默认回调）

```rust
let agent = AgentLoop::new(
    provider,
    config,
    cron_service,
    subagent_manager,
    tools_config,
).await?;

// run 方法内部会自动创建默认进度回调（ChannelProgressTracker）
agent.run(inbound_rx, outbound_tx).await?;
```

#### 场景 2：直接调用模式（自定义闭包）

```rust
use nanobot_agent::ProgressTracker;

// 自定义进度回调（闭包直接实现 trait）
let callback = |content: String, is_tool_hint: bool| {
    Box::pin(async move {
        println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
        Ok(())  // 返回 Result<()>
    })
};

let result = agent.process_direct(
    "帮我分析这个文件",
    "cli:direct",
    None,
    None,
    Some(Arc::new(callback)),
).await?;
```

#### 场景 3：直接调用模式（使用 ChannelProgressTracker）

```rust
let (tx, mut rx) = mpsc::channel(10);
let tracker = Arc::new(ChannelProgressTracker {
    tx,
    channel: "cli".to_string(),
    chat_id: "direct".to_string(),
});

// 启动接收任务
tokio::spawn(async move {
    while let Some(msg) = rx.recv().await {
        if msg.is_progress() {
            println!("[进度] {}", msg.content);
        }
    }
});

let result = agent.process_direct(
    "查询天气",
    "cli:direct",
    None,
    None,
    Some(tracker),
).await?;
```

## 功能需求列表

### 核心功能

#### 进度模块（新目录）

- [ ] **P0** 创建 `crates/agent/src/progress/mod.rs` 模块
- [ ] **P0** 定义 `ProgressTracker` trait（使用 `async_trait`）
- [ ] **P0** 实现 `ChannelProgressTracker` 结构体（默认实现，通过消息通道发送）
- [ ] **P0** 为闭包类型 `F` 直接实现 `ProgressTracker` trait
- [ ] **P0** 在 `crates/agent/src/lib.rs` 中导出 progress 模块
- [ ] **P0** 创建 `crates/agent/src/progress/tests.rs` 测试文件

#### AgentLoop 方法修改

- [ ] **P0** 修改 `re_act` 方法签名，添加 `on_progress` 参数
- [ ] **P0** 在 `re_act` 方法中集成进度通知逻辑
  - [ ] P0 发送思考内容（如果有）
  - [ ] P0 发送工具调用提示
- [ ] **P0** 实现 `format_tool_hint` 方法（格式化工具调用为简洁提示）
- [ ] **P0** 实现 `strip_think` 方法（清理思考内容中的特殊标记）
- [ ] **P0** 修改 `run` 方法，在内部创建 `ChannelProgressTracker` 并传递给 `re_act`
- [ ] **P0** 修改 `process_direct` 方法签名，添加 `on_progress` 参数

#### 测试

- [ ] **P0** 为 `ProgressTracker` trait 编写单元测试
  - [ ] P0 测试 `ChannelProgressTracker` 正确发送进度消息
  - [ ] P0 测试闭包直接实现 trait 的正确调用
- [ ] **P0** 为 `re_act` 方法编写集成测试
  - [ ] P0 测试进度回调被正确调用
  - [ ] P0 测试工具提示格式化逻辑
  - [ ] P0 测试思考内容的清理逻辑
- [ ] **P1** 测试回调执行失败不影响主流程
- [ ] **P1** 测试进度消息的正确路由（channel/chat_id）

### 扩展功能

- [ ] **P1** 提供进度消息过滤工具（过滤 `_progress` 消息）
- [ ] **P1** 添加进度消息节流机制（避免发送过于频繁）
- [ ] **P2** 支持自定义 `ProgressTracker` 实现的注册和发现机制

## 非功能需求

### 性能

- **低延迟**：进度通知不应显著增加处理延迟，回调执行时间应 < 1ms
- **异步非阻塞**：使用异步回调，避免阻塞主处理流程
- **内存效率**：使用 `Arc` 共享 notifier，避免不必要的拷贝

### 安全

- **错误隔离**：`track()` 方法返回 `Result<()>`，调用方使用 `if let Err(e) = tracker.track(...).await { error!(...) }` 捕获错误，不中断主流程
- **线程安全**：`ProgressTracker` trait 需要 `Send + Sync`，支持跨线程传递

### 兼容性

- **向后兼容**：`on_progress` 参数为 `Option`，不传入时不影响现有功能
- **API 稳定**：`new` 和 `run` 方法签名保持不变
- **Python 一致性**：进度消息格式与 Python 版本保持一致（metadata 字段）
- **设计一致性**：`on_progress` 作为方法参数传递，与 Python 版本架构一致

### 可维护性

- **代码规范**：遵循 AGENTS.md 中的开发规范
  - 使用 `thiserror` 定义错误类型（如果需要）
  - 测试代码与源代码分离
  - 使用 `///` 编写文档注释
- **文档完善**：为所有公共 API 编写文档，包括参数说明、返回值、示例
- **模块化**：进度通知功能独立为 `progress` 模块

### 测试要求

- **单元测试**：覆盖核心逻辑
  - [ ] P0 测试 `ChannelProgressTracker` 发送消息逻辑
  - [ ] P0 测试闭包直接实现 trait 的调用逻辑
  - [ ] P0 测试工具提示格式化逻辑（`format_tool_hint`）
  - [ ] P0 测试思考内容的清理逻辑（`strip_think`）
  - [ ] P1 测试回调失败不影响主流程
  
- **集成测试**：覆盖端到端场景
  - [ ] P0 测试 `run` 方法发送进度消息到 outbound_tx
  - [ ] P0 测试 `process_direct` 支持自定义回调
  - [ ] P1 测试进度消息的正确路由（channel/chat_id）
  
- **测试组织**：遵循项目规范
  - 测试代码放在 `tests.rs` 文件中
  - 测试命名使用描述性名称，不使用 `test_` 前缀
  - 使用 `#[cfg(test)] mod tests;` 引入测试模块

## 边界与不做事项

### 不在本次范围内

1. **进度消息的持久化**：进度消息不保存到会话历史中
2. **进度消息的重发**：不实现进度消息的可靠性保证（如重试机制）
3. **进度消息的优先级**：不实现消息优先级队列
4. **前端展示逻辑**：仅提供消息发送机制，不涉及 UI 展示

### 明确边界

1. **触发条件**：仅在工具调用时发送进度，普通文本响应不触发
2. **消息类型**：仅支持两种进度类型（思考内容、工具提示）
3. **实现范围**：仅修改 `AgentLoop`，不涉及其他 crate

## 假设与约束

### 技术假设

- **异步运行时**：使用 Tokio 作为异步运行时
- **消息通道**：使用 `mpsc` 通道进行消息传递
- **Rust 版本**：Rust >= 1.93（遵循 AGENTS.md 要求）
- **依赖库**：
  - `tokio`：异步运行时和通道
  - `tracing`：日志记录
  - `nanobot-channels`：消息类型定义

### 资源约束

- **开发时间**：预计 2-3 个工作日完成核心功能
- **人力投入**：单人开发，需要 Code Review

### 环境约束

- **CI/CD**：需通过 `cargo test` 和 `cargo clippy` 检查
- **代码规范**：需符合 `cargo fmt` 格式化要求
- **文档要求**：所有公共 API 必须有文档注释

## 实现路径建议

### 阶段 1：基础设施（P0）

1. 创建 `crates/agent/src/progress/mod.rs` 模块
2. 定义 `ProgressTracker` trait
3. 实现 `ChannelProgressTracker` 默认实现
4. 为闭包类型 `F` 直接实现 `ProgressTracker` trait
5. 在 `lib.rs` 中导出 progress 模块
6. 创建 `crates/agent/src/progress/tests.rs` 测试文件
7. 编写 progress 模块的单元测试

### 阶段 2：辅助方法（P0）

1. 实现 `strip_think` 方法（清理思考内容）
2. 实现 `format_tool_hint` 方法（格式化工具调用提示）
3. 编写辅助方法的单元测试

### 阶段 3：核心集成（P0）

1. 修改 `re_act` 方法签名，添加 `on_progress` 参数
2. 在 `re_act` 方法中集成进度通知逻辑
3. 修改 `run` 方法，在内部创建 `ChannelProgressTracker`
4. 修改 `process_direct` 方法签名，添加 `on_progress` 参数
5. 编写集成测试

### 阶段 4：文档与优化（P1）

1. 完善 API 文档
2. 添加使用示例到文档注释
3. 性能优化（如节流机制，可选）
4. Code Review 和测试覆盖

## 与 Python 版本对应关系

| Python | Rust | 说明 |
|--------|------|------|
| `on_progress` 作为 `_run_agent_loop` 参数 | `on_progress` 作为 `re_act` 参数 | ✅ 架构一致 |
| `_bus_progress` 在 `_process_message` 内部定义 | `ChannelProgressTracker` 在 `run` 方法内部创建 | ✅ 实现一致 |
| `process_direct` 支持自定义 `on_progress` | `process_direct` 支持自定义 `on_progress` | ✅ API 一致 |
| 默认使用 `_bus_progress` 或用户传入 | 默认使用 `ChannelProgressTracker` 或用户传入 | ✅ 行为一致 |
| 回调函数直接传入 | 闭包直接实现 trait，无需适配器 | ✅ 更简洁 |

## 参考文档

- Python 版本实现：`_nanobot/nanobot/agent/loop.py`
- 项目开发规范：`AGENTS.md`
- 消息类型定义：`crates/channels/src/messages/mod.rs`
- AgentLoop 实现：`crates/agent/src/loop/mod.rs`
