# 原生 async trait vs async-trait crate 评估

## 背景

项目要求 **Rust >= 1.93**，而 Rust 1.75 已经稳定支持原生 async trait。需要评估是否可以使用标准库的 async trait 来替代 `async-trait` crate。

---

## 方案对比

### 方案 1：使用 async-trait crate

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ProgressNotifier: Send + Sync {
    async fn notify(&self, content: String, is_tool_hint: bool);
}

// 展开后：
pub trait ProgressNotifier: Send + Sync {
    fn notify<'life0, 'async_trait>(
        &'life0 self,
        content: String,
        is_tool_hint: bool,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait;
}
```

### 方案 2：使用原生 async trait（Rust 1.75+）

```rust
pub trait ProgressNotifier: Send + Sync {
    async fn notify(&self, content: String, is_tool_hint: bool);
}

// 等价于：
pub trait ProgressNotifier: Send + Sync {
    fn notify(&self, content: String, is_tool_hint: bool) 
        -> impl Future<Output = ()> + Send;
}
```

---

## 关键差异

### 1. 动态分发（dyn trait）支持

**async-trait crate:**
```rust
// ✅ 直接支持
let notifier: Arc<dyn ProgressNotifier> = Arc::new(ChannelProgressNotifier { ... });
notifier.notify(content, is_tool_hint).await;  // 正常工作
```

**原生 async trait:**
```rust
// ❌ 直接使用会报错
let notifier: Arc<dyn ProgressNotifier> = Arc::new(ChannelProgressNotifier { ... });
notifier.notify(content, is_tool_hint).await;
// Error: the trait `ProgressNotifier` cannot be made into an object
// because async method `notify` returns `impl Future`
```

**原因**：原生 async trait 返回 `impl Future`，这是 RPIT（Return Position Impl Trait），目前**不支持 trait object**。

### 2. 解决方案对比

| 方案 | dyn 支持 | 代码复杂度 | 性能 |
|------|---------|-----------|------|
| async-trait crate | ✅ 开箱即用 | 简单 | 略有 Box 开销 |
| 原生 async trait + 手动 Box | ✅ 需手动实现 | 复杂 | 相同 |
| 原生 async trait + 泛型 | ❌ 不支持 dyn | 中等 | 略优（无 Box） |

#### 原生 async trait + 手动 Box

```rust
// 需要手动将返回类型改为 Pin<Box<dyn Future>>
pub trait ProgressNotifier: Send + Sync {
    fn notify(&self, content: String, is_tool_hint: bool) 
        -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

// 实现时也需要手动 Box
impl ProgressNotifier for ChannelProgressNotifier {
    fn notify(&self, content: String, is_tool_hint: bool) 
        -> Pin<Box<dyn Future<Output = ()> + Send + '_>> 
    {
        Box::pin(async move {
            // ...
        })
    }
}

// 闭包实现也需要手动 Box
impl<F> ProgressNotifier for F
where
    F: Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
{
    fn notify(&self, content: String, is_tool_hint: bool) 
        -> Pin<Box<dyn Future<Output = ()> + Send + '_>> 
    {
        self(content, is_tool_hint)
    }
}
```

**问题**：这本质上就是 async-trait crate 所做的事情，但需要手动编写。

---

## 项目现状分析

### 当前项目已使用 async-trait

```rust
// crates/tools/src/core.rs
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, ctx: &ToolContext, params: serde_json::Value) -> ToolResult;
}

// crates/provider/src/base/mod.rs
#[async_trait::async_trait]
pub trait Provider: Send + Sync + Clone + 'static {
    async fn chat(&self, messages: &[Message], options: &Options) -> Result<Message>;
}
```

**结论**：项目已经在使用 `async-trait` crate，且用法与 `ProgressNotifier` 类似（都需要 `dyn` 支持）。

---

## 推荐：继续使用 async-trait crate

### 理由

1. **一致性**：与现有 `Tool` 和 `Provider` trait 保持一致
2. **简单**：无需手动处理 `Pin<Box<...>>`，代码更简洁
3. **成熟**：async-trait crate 久经考验，生态支持完善
4. **性能**：Box 开销在进度通知场景下可忽略（非热点路径）
5. **dyn 支持**：开箱即用，无需额外处理

### 代码示例

```rust
use async_trait::async_trait;

/// 进度通知器
#[async_trait]
pub trait ProgressNotifier: Send + Sync {
    /// 发送进度通知
    async fn notify(&self, content: String, is_tool_hint: bool);
}

/// 通过消息通道发送进度
pub struct ChannelProgressNotifier {
    tx: mpsc::Sender<OutboundMessage>,
    channel: String,
    chat_id: String,
}

#[async_trait]
impl ProgressNotifier for ChannelProgressNotifier {
    async fn notify(&self, content: String, is_tool_hint: bool) {
        let mut msg = OutboundMessage::progress(content, is_tool_hint);
        msg.channel = self.channel.clone();
        msg.chat_id = self.chat_id.clone();
        
        if let Err(e) = self.tx.send(msg).await {
            error!("发送进度消息失败: {}", e);
        }
    }
}

/// 为闭包直接实现 trait
#[async_trait]
impl<F> ProgressNotifier for F
where
    F: Fn(String, bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
{
    async fn notify(&self, content: String, is_tool_hint: bool) {
        self(content, is_tool_hint).await;
    }
}
```

---

## 替代方案：未来迁移到原生 async trait

### 条件

当以下条件满足时，可以考虑迁移：

1. **Rust 支持 async trait 的 dyn**（目前仍在开发中）
   - 相关 RFC: [rfcs#3185](https://github.com/rust-lang/rfcs/pull/3185)
   - 预计在未来的 Rust 版本中支持

2. **项目统一迁移**
   - 同时迁移 `Tool`、`Provider`、`ProgressNotifier` 等所有 trait
   - 保持代码风格一致

### 迁移步骤（未来）

```rust
// 当 Rust 支持后
pub trait ProgressNotifier: Send + Sync {
    async fn notify(&self, content: String, is_tool_hint: bool);
}

// 直接使用 dyn
let notifier: Arc<dyn ProgressNotifier> = Arc::new(ChannelProgressNotifier { ... });
notifier.notify(content, is_tool_hint).await;
```

---

## 结论

| 方面 | async-trait crate | 原生 async trait |
|------|------------------|-----------------|
| dyn 支持 | ✅ 开箱即用 | ❌ 需手动 Box |
| 项目一致性 | ✅ 与现有代码一致 | ❌ 不一致 |
| 代码简洁 | ✅ 简洁 | ❌ 复杂 |
| 依赖成本 | 额外依赖（项目已有） | 无额外依赖 |
| 性能 | Box 开销（可忽略） | 无 Box 开销 |
| 未来兼容 | 易于迁移到原生 | 原生支持 |

**推荐**：继续使用 `async-trait` crate，与项目现有 `Tool` 和 `Provider` trait 保持一致。未来当 Rust 原生支持 async trait 的 dyn 时，再统一迁移所有 trait。
