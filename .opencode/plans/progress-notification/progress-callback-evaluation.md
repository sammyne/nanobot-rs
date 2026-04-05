# ProgressCallback 设计方案评估

## 背景

需要为 AgentLoop 实现进度通知功能，当前有两种设计方案：
1. **类型别名（Type Alias）** - 使用 `Arc<dyn Fn...>` 
2. **Trait** - 使用 `#[async_trait]` 定义 trait

本文档对比分析两种方案的优缺点，给出推荐建议。

---

## 方案 1：类型别名（Type Alias）

### 定义

```rust
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

/// 进度通知回调函数类型
/// 
/// # 参数
/// * `content` - 进度内容
/// * `is_tool_hint` - 是否为工具提示
pub type ProgressCallback = Arc<
    dyn Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync
>;
```

### 使用示例

```rust
// 场景 1：使用闭包
let callback = Arc::new(|content: String, is_tool_hint: bool| {
    Box::pin(async move {
        println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
    })
});

// 场景 2：使用函数
async fn my_progress_handler(content: String, is_tool_hint: bool) {
    println!("Progress: {}", content);
}

let callback = Arc::new(|content, is_tool_hint| {
    Box::pin(my_progress_handler(content, is_tool_hint))
});

// 场景 3：捕获外部变量
let tx = outbound_tx.clone();
let callback = Arc::new(move |content: String, is_tool_hint: bool| {
    let tx = tx.clone();
    Box::pin(async move {
        let msg = OutboundMessage::progress(content, is_tool_hint);
        tx.send(msg).await.ok();
    })
});
```

### 优点

✅ **简洁性**
- 一行类型定义即可，无需额外的 trait 定义
- 使用时直接传入闭包，代码简洁

✅ **Python 一致性**
- 与 Python 版本的回调函数模式完全一致
- 移植成本低，理解成本低

✅ **灵活性高**
- 支持闭包、函数指针等多种形式
- 可以捕获外部变量（如 `outbound_tx`）
- 无需为每种实现定义新的类型

✅ **无额外复杂度**
- 不引入新的泛型参数到 `AgentLoop`
- API 简单：`Option<ProgressCallback>`

### 缺点

❌ **类型复杂**
- 类型签名较长，不够直观
- 新手可能需要时间理解 `Pin<Box<dyn Future>>`

❌ **性能略低**
- 每次调用都需要动态分发（dyn dispatch）
- 但实际性能影响微乎其微（进度通知不是热点路径）

❌ **难以扩展**
- 如果未来需要添加更多方法，需要修改类型签名
- 但当前场景下，进度通知只需要一个方法

---

## 方案 2：Trait（使用 async-trait）

### 定义

```rust
use async_trait::async_trait;

/// 进度通知器 trait
#[async_trait]
pub trait ProgressNotifier: Send + Sync {
    /// 发送进度通知
    /// 
    /// # 参数
    /// * `content` - 进度内容
    /// * `is_tool_hint` - 是否为工具提示
    async fn notify(&self, content: String, is_tool_hint: bool);
}
```

### 使用示例

```rust
// 场景 1：定义具体类型
struct ConsoleNotifier;

#[async_trait]
impl ProgressNotifier for ConsoleNotifier {
    async fn notify(&self, content: String, is_tool_hint: bool) {
        println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
    }
}

let notifier = Arc::new(ConsoleNotifier);

// 场景 2：定义带状态的类型
struct BusNotifier {
    tx: mpsc::Sender<OutboundMessage>,
    channel: String,
    chat_id: String,
}

#[async_trait]
impl ProgressNotifier for BusNotifier {
    async fn notify(&self, content: String, is_tool_hint: bool) {
        let mut msg = OutboundMessage::progress(content, is_tool_hint);
        msg.channel = self.channel.clone();
        msg.chat_id = self.chat_id.clone();
        self.tx.send(msg).await.ok();
    }
}

let notifier = Arc::new(BusNotifier {
    tx: outbound_tx,
    channel: "cli".to_string(),
    chat_id: "direct".to_string(),
});

// 场景 3：使用闭包（通过适配器）
struct ClosureNotifier<F>(F);

#[async_trait]
impl<F> ProgressNotifier for ClosureNotifier<F>
where
    F: Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
{
    async fn notify(&self, content: String, is_tool_hint: bool) {
        (self.0)(content, is_tool_hint).await;
    }
}
```

### 在 AgentLoop 中的使用

#### 方案 2A：泛型参数

```rust
pub struct AgentLoop<P: Provider, N: ProgressNotifier> {
    provider: P,
    // ... 其他字段 ...
    on_progress: Option<Arc<N>>,
}

impl<P: Provider, N: ProgressNotifier> AgentLoop<P, N> {
    pub async fn new(
        provider: P,
        config: AgentDefaults,
        // ... 其他参数 ...
        on_progress: Option<Arc<N>>,
    ) -> Result<Self> {
        // ...
    }
}
```

**问题**：
- 增加了泛型参数，类型签名变复杂
- 如果用户不使用进度通知，仍需指定类型：`AgentLoop<MyProvider, ()>` 或使用 `Option<Arc<dyn ProgressNotifier>>`

#### 方案 2B：Trait Object（推荐）

```rust
pub struct AgentLoop<P: Provider> {
    provider: P,
    // ... 其他字段 ...
    on_progress: Option<Arc<dyn ProgressNotifier>>,
}
```

**优点**：不增加泛型参数

### 优点

✅ **类型清晰**
- trait 定义直观，语义明确
- 方法命名清晰（`notify`），代码可读性好

✅ **易于扩展**
- 未来可以添加更多方法（如 `on_start`, `on_complete`）
- 可以添加关联类型、常量等

✅ **符合 Rust 惯用模式**
- Rust 中常用 trait 定义行为
- 便于类型系统约束和静态分析

✅ **可测试性强**
- 可以轻松创建 Mock 实现用于测试
- 示例：
  ```rust
  #[cfg(test)]
  struct MockNotifier {
      calls: Arc<Mutex<Vec<(String, bool)>>>,
  }
  
  #[async_trait]
  impl ProgressNotifier for MockNotifier {
      async fn notify(&self, content: String, is_tool_hint: bool) {
          self.calls.lock().await.push((content, is_tool_hint));
      }
  }
  ```

### 缺点

❌ **代码量稍多**
- 需要为每种实现定义具体类型
- 或使用闭包适配器（增加间接层）

❌ **使用复杂度**
- 简单场景下不如闭包直接
- 需要定义类型或使用适配器

❌ **动态分发**
- 使用 `dyn ProgressNotifier` 时仍有动态分发开销
- 与类型别名方案性能相同

---

## 对比总结表

| 维度 | 类型别名 | Trait (方案 2B) |
|------|---------|----------------|
| **定义复杂度** | ⭐⭐⭐⭐⭐ 一行定义 | ⭐⭐⭐⭐ 需要 trait 定义 |
| **使用简洁性** | ⭐⭐⭐⭐⭐ 直接传闭包 | ⭐⭐⭐ 需定义类型或适配器 |
| **类型清晰度** | ⭐⭐ 类型签名复杂 | ⭐⭐⭐⭐⭐ 语义清晰 |
| **扩展性** | ⭐⭐ 需修改类型 | ⭐⭐⭐⭐⭐ 易于扩展 |
| **Python 一致性** | ⭐⭐⭐⭐⭐ 完全一致 | ⭐⭐⭐ 需适配 |
| **可测试性** | ⭐⭐⭐ 可用闭包 Mock | ⭐⭐⭐⭐⭐ 可定义 Mock 类型 |
| **性能** | ⭐⭐⭐⭐ 动态分发 | ⭐⭐⭐⭐ 动态分发 |
| **符合 Rust 惯例** | ⭐⭐⭐ 回调模式 | ⭐⭐⭐⭐⭐ trait 模式 |

---

## 推荐方案

### 🎯 推荐：**方案 2B（Trait Object）**

虽然类型别名方案更简洁，但考虑到以下因素，推荐使用 Trait 方案：

#### 理由

1. **项目已有 `async-trait` 依赖**
   - 不引入额外依赖成本

2. **更好的扩展性**
   - 未来可能需要：
     - 添加更多生命周期钩子（`on_start`, `on_complete`）
     - 添加配置项（如日志级别、过滤规则）
     - 添加关联类型（如自定义元数据）
   - Trait 更容易扩展这些功能

3. **更好的可测试性**
   - 可以定义专门的 Mock 类型
   - 测试代码更清晰

4. **符合 Rust 生态习惯**
   - Rust 中行为抽象通常使用 trait
   - 便于与其他库集成

5. **类型清晰度**
   - `dyn ProgressNotifier` 比 `Arc<dyn Fn...Pin<Box...>>` 更易理解
   - 方法名 `notify` 语义明确

#### 实现建议

```rust
// crates/agent/src/progress.rs (新文件)
use async_trait::async_trait;

/// 进度通知器
#[async_trait]
pub trait ProgressNotifier: Send + Sync {
    /// 发送进度通知
    async fn notify(&self, content: String, is_tool_hint: bool);
}

/// 通过消息通道发送进度的默认实现
pub struct BusNotifier {
    tx: mpsc::Sender<OutboundMessage>,
    channel: String,
    chat_id: String,
}

#[async_trait]
impl ProgressNotifier for BusNotifier {
    async fn notify(&self, content: String, is_tool_hint: bool) {
        let mut msg = OutboundMessage::progress(content, is_tool_hint);
        msg.channel = self.channel.clone();
        msg.chat_id = self.chat_id.clone();
        
        if let Err(e) = self.tx.send(msg).await {
            error!("发送进度消息失败: {}", e);
        }
    }
}

/// 闭包适配器（简化使用）
pub struct ClosureNotifier<F>(pub F);

#[async_trait]
impl<F> ProgressNotifier for ClosureNotifier<F>
where
    F: Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
{
    async fn notify(&self, content: String, is_tool_hint: bool) {
        (self.0)(content, is_tool_hint).await;
    }
}
```

```rust
// crates/agent/src/loop/mod.rs
use crate::progress::{ProgressNotifier, BusNotifier};

pub struct AgentLoop<P: Provider> {
    provider: P,
    // ... 其他字段 ...
    on_progress: Option<Arc<dyn ProgressNotifier>>,
}

impl<P: Provider> AgentLoop<P> {
    pub async fn new(
        provider: P,
        config: AgentDefaults,
        cron_service: Option<Arc<CronService>>,
        subagent_manager: Option<Arc<SubagentManager<P>>>,
        tools_config: nanobot_config::ToolsConfig,
        on_progress: Option<Arc<dyn ProgressNotifier>>, // 新增
    ) -> Result<Self> {
        // ...
    }
    
    /// 创建默认的进度通知器
    pub fn create_default_notifier(
        outbound_tx: mpsc::Sender<OutboundMessage>,
        channel: String,
        chat_id: String,
    ) -> Arc<dyn ProgressNotifier> {
        Arc::new(BusNotifier {
            tx: outbound_tx,
            channel,
            chat_id,
        })
    }
}
```

```rust
// 使用示例

// 场景 1：使用默认实现
let notifier = AgentLoop::create_default_notifier(
    outbound_tx, 
    "cli".to_string(), 
    "direct".to_string()
);
let agent = AgentLoop::new(provider, config, None, None, tools_config, Some(notifier)).await?;

// 场景 2：使用闭包（通过适配器）
use nanobot_agent::progress::ClosureNotifier;
let notifier = Arc::new(ClosureNotifier(|content, is_tool_hint| {
    Box::pin(async move {
        println!("[Progress] {}", content);
    })
}));
let agent = AgentLoop::new(provider, config, None, None, tools_config, Some(notifier)).await?;

// 场景 3：自定义实现
struct MyNotifier;
#[async_trait]
impl ProgressNotifier for MyNotifier {
    async fn notify(&self, content: String, is_tool_hint: bool) {
        // 自定义逻辑
    }
}
let agent = AgentLoop::new(provider, config, None, None, tools_config, Some(Arc::new(MyNotifier))).await?;
```

---

## 备选方案：混合模式

如果想要同时兼顾简洁性和扩展性，可以采用混合模式：

```rust
// 定义 trait
#[async_trait]
pub trait ProgressNotifier: Send + Sync {
    async fn notify(&self, content: String, is_tool_hint: bool);
}

// 提供类型别名，简化使用
pub type ProgressCallback = Arc<dyn ProgressNotifier>;

// 提供便捷的构造函数
impl dyn ProgressNotifier {
    pub fn from_closure<F>(f: F) -> Arc<dyn ProgressNotifier>
    where
        F: Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        Arc::new(ClosureNotifier(f))
    }
}

// 使用
let callback = ProgressNotifier::from_closure(|content, _| {
    Box::pin(async { println!("{}", content); })
});
```

---

## 结论

**推荐使用 Trait 方案（方案 2B）**，理由：

1. ✅ 项目已有 `async-trait`，无额外成本
2. ✅ 扩展性强，未来易于添加功能
3. ✅ 类型清晰，符合 Rust 惯例
4. ✅ 可测试性好
5. ✅ 可通过 `ClosureNotifier` 适配器支持闭包使用方式

如果项目后续发现闭包使用场景占主导，可以考虑混合模式，提供便捷的闭包构造函数。

---

## Python 版本关键设计

**重要**：Python 版本中 `on_progress` 是**方法参数**，而非实例字段。

```python
# ✅ Python 版本：on_progress 作为方法参数
async def _run_agent_loop(
    self,
    initial_messages: list[dict],
    on_progress: Callable[..., Awaitable[None]] | None = None,  # 方法参数
) -> tuple[str | None, list[str], list[dict]]:
    # ...

# run 方法中创建默认回调并传递
async def _process_message(self, msg: InboundMessage, on_progress=None):
    async def _bus_progress(content: str, *, tool_hint: bool = False) -> None:
        # 默认实现...
    
    final_content, _, all_msgs = await self._run_agent_loop(
        initial_messages, on_progress=on_progress or _bus_progress,  # 传递给方法
    )
```

---

## 调整后的 Rust 设计（与 Python 一致）

### 设计原则

✅ **`on_progress` 不作为 `AgentLoop` 字段**
✅ **作为方法参数传递**
✅ **在方法内部创建默认实现**

### 实现

```rust
// crates/agent/src/progress.rs
use async_trait::async_trait;
use tokio::sync::mpsc;

/// 进度通知器
#[async_trait]
pub trait ProgressNotifier: Send + Sync {
    /// 发送进度通知
    async fn notify(&self, content: String, is_tool_hint: bool);
}

/// 通过消息通道发送进度的默认实现
pub struct BusNotifier {
    tx: mpsc::Sender<OutboundMessage>,
    channel: String,
    chat_id: String,
}

#[async_trait]
impl ProgressNotifier for BusNotifier {
    async fn notify(&self, content: String, is_tool_hint: bool) {
        let mut msg = OutboundMessage::progress(content, is_tool_hint);
        msg.channel = self.channel.clone();
        msg.chat_id = self.chat_id.clone();
        
        if let Err(e) = self.tx.send(msg).await {
            error!("发送进度消息失败: {}", e);
        }
    }
}

/// 闭包适配器（简化使用）
pub struct ClosureNotifier<F>(pub F);

#[async_trait]
impl<F> ProgressNotifier for ClosureNotifier<F>
where
    F: Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
{
    async fn notify(&self, content: String, is_tool_hint: bool) {
        (self.0)(content, is_tool_hint).await;
    }
}
```

```rust
// crates/agent/src/loop/mod.rs
use crate::progress::{ProgressNotifier, BusNotifier};

pub struct AgentLoop<P: Provider> {
    provider: P,
    config: AgentDefaults,
    sessions: Arc<SessionManager>,
    tool_registry: ToolRegistry,
    context: ContextBuilder,
    consolidating: Arc<Mutex<HashSet<String>>>,
    // ❌ 不添加 on_progress 字段
}

impl<P: Provider> AgentLoop<P> {
    /// ✅ new 方法签名保持不变
    pub async fn new(
        provider: P,
        config: AgentDefaults,
        cron_service: Option<Arc<CronService>>,
        subagent_manager: Option<Arc<SubagentManager<P>>>,
        tools_config: nanobot_config::ToolsConfig,
    ) -> Result<Self> {
        // ... 现有逻辑不变 ...
    }
    
    /// ✅ re_act 方法添加 on_progress 参数
    pub async fn re_act(
        &self,
        messages: Vec<Message>,
        channel: &str,
        chat_id: &str,
        on_progress: Option<Arc<dyn ProgressNotifier>>,  // 方法参数
    ) -> Result<ReActResult> {
        // ... 现有逻辑 ...
        
        if !tool_calls.is_empty() {
            // 发送进度通知
            if let Some(ref notifier) = on_progress {
                // 1. 发送思考内容（如果有）
                if !content.is_empty() {
                    notifier.notify(content.clone(), false).await;
                }
                
                // 2. 发送工具提示
                let hint = self.format_tool_hint(&tool_calls);
                notifier.notify(hint, true).await;
            }
            
            // ... 执行工具 ...
        }
        
        // ...
    }
    
    /// ✅ run 方法在内部创建默认回调
    pub async fn run(
        &self,
        inbound_rx: mpsc::Receiver<InboundMessage>,
        outbound_tx: mpsc::Sender<OutboundMessage>,
    ) -> Result<()> {
        while let Some(msg) = inbound_rx.recv().await {
            // 在消息处理内部创建默认进度回调（类似 Python 的 _bus_progress）
            let default_notifier = Arc::new(BusNotifier {
                tx: outbound_tx.clone(),
                channel: msg.channel.clone(),
                chat_id: msg.chat_id.clone(),
            });
            
            // 调用 re_act，传入默认回调
            let result = self.re_act(
                messages,
                &msg.channel,
                &msg.chat_id,
                Some(default_notifier),  // 传递默认实现
            ).await?;
        }
        Ok(())
    }
    
    /// ✅ process_direct 方法支持自定义回调
    pub async fn process_direct(
        &self,
        content: &str,
        session_key: &str,
        channel: Option<&str>,
        chat_id: Option<&str>,
        on_progress: Option<Arc<dyn ProgressNotifier>>,  // 方法参数
    ) -> Result<String> {
        let channel = channel.unwrap_or("cli");
        let chat_id = chat_id.unwrap_or("direct");
        
        let result = self.re_act(
            messages,
            channel,
            chat_id,
            on_progress,  // 直接传递用户的回调
        ).await?;
        
        Ok(result.content)
    }
}
```

---

## 使用示例

### 场景 1：交互式模式（使用默认回调）

```rust
let agent = AgentLoop::new(
    provider,
    config,
    cron_service,
    subagent_manager,
    tools_config,
    // ❌ 不再需要传递 on_progress
).await?;

// run 方法内部会自动创建默认进度回调
agent.run(inbound_rx, outbound_tx).await?;
```

### 场景 2：直接调用模式（自定义回调）

```rust
let custom_notifier = Arc::new(ClosureNotifier(|content, is_tool_hint| {
    Box::pin(async move {
        println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
    })
}));

let result = agent.process_direct(
    "帮我分析这个文件",
    "cli:direct",
    None,
    None,
    Some(custom_notifier),  // 传递自定义回调
).await?;
```

### 场景 3：直接调用模式（无进度通知）

```rust
let result = agent.process_direct(
    "查询天气",
    "cli:direct",
    None,
    None,
    None,  // 不使用进度通知
).await?;
```

---

## 与 Python 版本的对应关系

| Python | Rust |
|--------|------|
| `on_progress` 作为 `_run_agent_loop` 参数 | `on_progress` 作为 `re_act` 参数 ✅ |
| `_bus_progress` 在 `_process_message` 内部定义 | `BusNotifier` 在 `run` 方法内部创建 ✅ |
| `process_direct` 支持自定义 `on_progress` | `process_direct` 支持自定义 `on_progress` ✅ |
| 默认使用 `_bus_progress` 或用户传入 | 默认使用 `BusNotifier` 或用户传入 ✅ |

---

## 影响分析

### 对现有代码的影响

**最小化影响的实现路径**：

1. `AgentLoop::new` 方法签名**不变** ✅
   ```rust
   // ✅ 签名保持不变，无需修改调用方
   pub async fn new(
       provider: P,
       config: AgentDefaults,
       cron_service: Option<Arc<CronService>>,
       subagent_manager: Option<Arc<SubagentManager<P>>>,
       tools_config: nanobot_config::ToolsConfig,
   ) -> Result<Self>
   ```

2. `re_act` 方法签名变更（内部方法）
   ```rust
   // 旧签名
   pub async fn re_act(&self, messages: Vec<Message>, channel: &str, chat_id: &str) -> Result<ReActResult>
   
   // 新签名
   pub async fn re_act(
       &self,
       messages: Vec<Message>,
       channel: &str,
       chat_id: &str,
       on_progress: Option<Arc<dyn ProgressNotifier>>,
   ) -> Result<ReActResult>
   ```

3. `run` 方法签名**不变**，内部创建默认回调 ✅
   ```rust
   pub async fn run(
       &self,
       inbound_rx: mpsc::Receiver<InboundMessage>,
       outbound_tx: mpsc::Sender<OutboundMessage>,
   ) -> Result<()>
   ```

4. `process_direct` 方法添加可选参数（向后兼容）
   ```rust
   // 旧签名
   pub async fn process_direct(&self, content: &str, ...) -> Result<String>
   
   // 新签名
   pub async fn process_direct(
       &self,
       content: &str,
       session_key: &str,
       channel: Option<&str>,
       chat_id: Option<&str>,
       on_progress: Option<Arc<dyn ProgressNotifier>>,
   ) -> Result<String>
   ```

### 需要修改的文件

1. **新增文件**：
   - `crates/agent/src/progress.rs` - 定义 trait 和默认实现
   - `crates/agent/src/progress/tests.rs` - 单元测试

2. **修改文件**：
   - `crates/agent/src/lib.rs` - 导出 progress 模块
   - `crates/agent/src/loop/mod.rs` - 修改 re_act、run、process_direct 方法
   - 其他调用 `re_act` 的地方（内部调用）

---

## 下一步行动

如果确认使用 Trait 方案，需要更新需求文档中的以下部分：

1. **类型定义**：将 `ProgressCallback` 类型别名改为 `ProgressNotifier` trait
2. **默认实现**：使用 `BusNotifier` 结构体
3. **使用示例**：更新为 trait 使用方式
4. **实现路径**：增加 "定义 progress 模块" 步骤

是否需要我更新需求文档？
