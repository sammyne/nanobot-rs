# TODO

## 任务列表

### 阶段 1：基础设施（ProgressTracker trait 和实现）

| 序号 | 优先级 | 任务描述 | 涉及文件 | 工作量 | 依赖项 | 风险/注意点 |
|------|--------|----------|----------|--------|--------|-------------|
| ✅ 1.1 | P0 | 创建 `progress/mod.rs` 模块文件 | `crates/agent/src/progress/mod.rs` (新增) | 0.5h | 无 | 需遵循项目模块组织规范 |
| ✅ 1.2 | P0 | 定义 `ProgressTracker` trait，包含 `async fn track(&self, content: String, is_tool_hint: bool) -> Result<()>` 方法 | `crates/agent/src/progress/mod.rs` (修改) | 0.5h | 1.1 | 需使用 `async_trait` 宏，trait 必须继承 `Send + Sync`，返回 `anyhow::Result<()>` 便于测试失败场景 |
| ✅ 1.3 | P0 | 实现 `ChannelProgressTracker` 结构体（包含 tx、channel、chat_id 字段） | `crates/agent/src/progress/mod.rs` (修改) | 1h | 1.2 | 发送失败时返回错误，使用 `anyhow!` 包装错误信息 |
| ✅ 1.4 | P0 | 为 `ChannelProgressTracker` 实现 `ProgressTracker` trait | `crates/agent/src/progress/mod.rs` (修改) | 1h | 1.3 | 需调用 `OutboundMessage::progress()` 构造消息，并设置 channel 和 chat_id |
| ✅ 1.5 | P0 | 为闭包类型 `F` 直接实现 `ProgressTracker` trait（无需适配器） | `crates/agent/src/progress/mod.rs` (修改) | 1h | 1.2 | 闭包签名为 `Fn(String, bool) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>`，需添加 `Send + Sync` 约束 |
| ✅ 1.6 | P0 | 在 `lib.rs` 中导出 progress 模块和公共类型 | `crates/agent/src/lib.rs` (修改) | 0.5h | 1.2-1.5 | 使用 `pub use crate::progress::{ProgressTracker, ChannelProgressTracker};` |
| ✅ 1.7 | P0 | 创建 `progress/tests.rs` 测试文件 | `crates/agent/src/progress/tests.rs` (新增) | 0.2h | 1.1 | 在 `mod.rs` 中添加 `#[cfg(test)] mod tests;` |
| ✅ 1.8 | P0 | 添加 `async_trait` 依赖到 `Cargo.toml` | `crates/agent/Cargo.toml` (修改) | 0.2h | 无 | 需在工作空间层面添加，成员 crate 使用 `workspace = true` |

**阶段 1 验收标准：**
- ✅ `ProgressTracker` trait 编译通过，具有正确的 `Send + Sync` 约束
- ✅ `ChannelProgressTracker` 可以成功创建并发送进度消息
- ✅ 闭包可以直接作为 `ProgressTracker` 使用，无需额外适配器
- ✅ 公共 API 可从 `nanobot_agent` crate 正确导出
- ✅ 测试文件 `progress/tests.rs` 创建完成并通过测试

**实际完成情况（阶段 1）：**
- 所有任务已完成
- 测试全部通过（6 个测试）
- clippy 检查通过

---

### 阶段 2：辅助方法（strip_think 和 format_tool_hint）

| 序号 | 优先级 | 任务描述 | 涉及文件 | 工作量 | 依赖项 | 风险/注意点 |
|------|--------|----------|----------|--------|--------|-------------|
| ✅ 2.1 | P0 | 实现 `strip_think` 静态方法（清理 `<think…</think〉` 标签） | `crates/agent/src/loop/mod.rs` (修改) | 1h | 无 | 参考 Python 版本使用正则表达式 `r"<think[\s\S]*?</think〉"`，需引入 `regex` crate |
| ✅ 2.2 | P0 | 实现 `format_tool_hint` 方法（格式化工具调用为简洁提示） | `crates/agent/src/loop/mod.rs` (修改) | 1h | 无 | 参考 Python 版本，格式为 `tool_name("arg...")`，参数超过 40 字符时截断 |
| ✅ 2.3 | P0 | 为 `strip_think` 编写单元测试 | `crates/agent/src/loop/tests.rs` (修改) | 1h | 2.1 | 测试用例：正常内容、包含 think 标签、空字符串、仅 think 标签、嵌套标签 |
| ✅ 2.4 | P0 | 为 `format_tool_hint` 编写单元测试 | `crates/agent/src/loop/tests.rs` (修改) | 1h | 2.2 | 测试用例：单个工具、多个工具、参数截断、空参数、特殊字符 |
| ✅ 2.5 | P0 | 添加 `regex` 依赖到 `Cargo.toml`（如果尚未添加） | `crates/agent/Cargo.toml` (修改) | 0.2h | 无 | 需在工作空间层面添加 |

**阶段 2 验收标准：**
- ✅ `strip_think` 可以正确清理 `<think…</think〉` 标签，返回清理后的字符串
- ✅ `format_tool_hint` 可以正确格式化工具调用列表，输出格式符合 Python 版本
- ✅ 所有测试用例通过，覆盖率 > 90%
- ✅ 方法签名为 `pub fn` 或 `fn`（根据是否需要外部调用决定）

---

### 阶段 3：核心集成（修改 AgentLoop 方法）

| 序号 | 优先级 | 任务描述 | 涉及文件 | 工作量 | 依赖项 | 风险/注意点 |
|------|--------|----------|----------|--------|--------|-------------|
| 3.1 | P0 | 修改 `re_act` 方法签名，添加 `on_progress: Option<Arc<dyn ProgressTracker>>` 参数 | `crates/agent/src/loop/mod.rs` (修改) | 0.5h | 1.2 | 需保持向后兼容，参数为 `Option`，默认为 `None` |
| 3.2 | P0 | 在 `re_act` 方法中，当检测到工具调用时发送进度通知 | `crates/agent/src/loop/mod.rs` (修改) | 2h | 3.1, 2.1, 2.2 | 需在 `if !tool_calls.is_empty()` 分支中调用进度通知，先发送思考内容（如果有），再发送工具提示 |
| 3.3 | P0 | 修改 `run` 方法，在消息处理循环中创建 `ChannelProgressTracker` | `crates/agent/src/loop/mod.rs` (修改) | 1.5h | 3.1, 1.3 | 在 `inbound_rx.recv().await` 的 `Some(msg)` 分支中创建 `ChannelProgressTracker`，并传递给 `process_message` |
| 3.4 | P0 | 重构 `process_message` 方法，添加 `on_progress` 参数 | `crates/agent/src/loop/mod.rs` (修改) | 2h | 3.1 | 需在 `process_message` 中添加 `on_progress` 参数，并传递给 `re_act`；`run` 方法传入默认实现，`process_direct` 传入用户自定义实现 |
| 3.5 | P0 | 修改 `process_direct` 方法签名，添加 `on_progress` 参数 | `crates/agent/src/loop/mod.rs` (修改) | 1h | 3.1, 3.4 | 参数为 `Option<Arc<dyn ProgressTracker>>`，允许用户传入自定义进度回调 |
| 3.6 | P0 | 更新 `process_system_message` 方法，支持进度通知 | `crates/agent/src/loop/mod.rs` (修改) | 1h | 3.4 | 需从 `process_message` 获取 `on_progress` 参数并传递给 `re_act` |
| 3.7 | P0 | 确保进度通知失败不影响主流程（捕获 `track()` 返回的错误并记录日志） | `crates/agent/src/loop/mod.rs` (修改) | 0.5h | 3.2 | 使用 `if let Err(e) = tracker.track(...).await { error!(...) }` 捕获错误，不中断处理流程 |

**阶段 3 验收标准：**
- ✅ `re_act` 方法正确接收 `on_progress` 参数，并在工具调用时发送进度通知
- ✅ `run` 方法在处理消息时自动创建并传递默认的 `ChannelProgressTracker`
- ✅ `process_direct` 方法支持用户传入自定义进度回调
- ✅ 进度消息通过 `outbound_tx` 正确发送，格式与 Python 版本一致
- ✅ 进度通知失败不会影响主流程，仅记录错误日志

---

### 阶段 4：测试与文档

| 序号 | 优先级 | 任务描述 | 涉及文件 | 工作量 | 依赖项 | 风险/注意点 |
|------|--------|----------|----------|--------|--------|-------------|
| 4.1 | P0 | 为 `ChannelProgressTracker` 编写单元测试（测试消息发送和失败场景） | `crates/agent/src/progress/tests.rs` (修改) | 1.5h | 1.3, 1.4 | 创建 `mpsc::channel`，验证发送的消息内容、metadata 字段、is_progress 和 is_tool_hint 标志；测试通道关闭时返回错误 |
| 4.2 | P0 | 为闭包直接实现 trait 编写单元测试 | `crates/agent/src/progress/tests.rs` (修改) | 1h | 1.5 | 验证闭包可以被正确调用，参数正确传递，返回 `Result<()>` |
| 4.3 | P0 | 为 `re_act` 方法编写集成测试（测试进度回调调用） | `crates/agent/src/loop/tests.rs` (修改) | 2h | 3.2 | Mock Provider 返回带工具调用的响应，验证 `on_progress` 被正确调用 |
| 4.4 | P0 | 为 `process_direct` 方法编写集成测试（测试自定义回调） | `crates/agent/src/loop/tests.rs` (修改) | 2h | 3.5 | 传入自定义闭包作为 `on_progress`，验证回调被触发 |
| 4.5 | P1 | 测试进度通知失败不影响主流程 | `crates/agent/src/loop/tests.rs` (修改) | 1.5h | 3.7 | 模拟进度通知失败（如通道关闭），验证主流程继续执行 |
| 4.6 | P1 | 测试进度消息的正确路由（channel/chat_id） | `crates/agent/src/loop/tests.rs` (修改) | 1h | 3.3 | 验证进度消息的 channel 和 chat_id 字段与原始消息一致 |
| 4.7 | P0 | 为 `ProgressTracker` trait 编写文档注释 | `crates/agent/src/progress/mod.rs` (修改) | 0.5h | 1.2 | 使用 `///` 编写文档，包含参数说明、返回值、示例 |
| 4.8 | P0 | 为 `ChannelProgressTracker` 编写文档注释 | `crates/agent/src/progress/mod.rs` (修改) | 0.5h | 1.3 | 包含使用示例 |
| 4.9 | P0 | 为 `re_act` 方法更新文档注释，说明 `on_progress` 参数 | `crates/agent/src/loop/mod.rs` (修改) | 0.5h | 3.1 | 更新现有文档，添加参数说明和使用示例 |
| 4.10 | P0 | 为 `process_direct` 方法更新文档注释，说明 `on_progress` 参数 | `crates/agent/src/loop/mod.rs` (修改) | 0.5h | 3.5 | 更新现有文档，添加参数说明和使用示例 |
| 4.11 | P1 | 在 `lib.rs` 中添加模块级文档，说明进度通知功能 | `crates/agent/src/lib.rs` (修改) | 0.5h | 1.6 | 添加功能概述和使用场景说明 |

**阶段 4 验收标准：**
- ✅ 所有单元测试和集成测试通过，覆盖率 > 85%
- ✅ 进度消息格式与 Python 版本一致（metadata 字段包含 `_progress` 和 `_tool_hint`）
- ✅ 进度通知失败不影响主流程
- ✅ 所有公共 API 有完整的文档注释，包含参数说明、返回值和示例
- ✅ `cargo doc --no-deps` 无警告

---

## 实现建议

### 技术方案

#### 依赖管理

在 `Cargo.toml` 中添加依赖（优先在工作空间层面管理）：

```toml
# 工作空间根 Cargo.toml
[workspace.dependencies]
async-trait = "0.1"
regex = "1.10"

# crates/agent/Cargo.toml
[dependencies]
async-trait.workspace = true
regex.workspace = true
```

#### 模块组织

遵循项目规范，测试代码与源代码分离：

```
crates/agent/src/
├── lib.rs           # 导出 progress 模块
├── progress/
│   ├── mod.rs       # ProgressTracker trait 和实现
│   └── tests.rs     # progress 模块测试
└── loop/
    ├── mod.rs       # AgentLoop 实现
    └── tests.rs     # AgentLoop 测试
```

#### 错误处理

- `ProgressTracker::track` 方法返回 `anyhow::Result<()>`，便于测试失败场景
- `ChannelProgressTracker` 发送失败时返回错误（使用 `anyhow!` 包装）
- 调用方使用 `if let Err(e) = tracker.track(...).await { error!(...) }` 捕获错误，不中断主流程
- 主流程不依赖进度通知的成功与否

### 测试策略

#### 单元测试

1. **progress 模块测试**（`crates/agent/src/progress/tests.rs`）：
   - `channel_tracker_sends_message`: 测试 `ChannelProgressTracker` 发送消息成功
   - `channel_tracker_sets_metadata`: 测试 metadata 字段正确设置
   - `channel_tracker_returns_error_on_closed_channel`: 测试通道关闭时返回错误
   - `closure_tracker_works`: 测试闭包直接实现 trait
   - `closure_tracker_receives_correct_args`: 测试参数正确传递
   - `closure_tracker_propagates_error`: 测试闭包返回错误时正确传播

2. **loop 模块测试**（`crates/agent/src/loop/tests.rs`）：
   - `strip_think_removes_tags`: 测试清理 think 标签
   - `strip_think_handles_empty`: 测试空字符串处理
   - `format_tool_hint_single`: 测试单个工具格式化
   - `format_tool_hint_multiple`: 测试多个工具格式化
   - `format_tool_hint_truncation`: 测试参数截断

#### 集成测试

1. **re_act 方法测试**：
   - `react_calls_progress_on_tool_calls`: 测试工具调用时触发进度通知
   - `react_sends_cleaned_content`: 测试发送清理后的思考内容
   - `react_sends_tool_hint`: 测试发送工具提示

2. **process_direct 方法测试**：
   - `process_direct_accepts_custom_callback`: 测试接受自定义回调
   - `process_direct_callback_receives_progress`: 测试回调收到进度消息

3. **错误场景测试**：
   - `progress_failure_does_not_break_main_flow`: 测试进度通知失败不影响主流程

### CI/CD 注意事项

#### 构建流程影响

- 无需修改 CI 配置，新增依赖会自动拉取
- 确保所有测试在 CI 环境通过：`cargo test --all-features`
- 确保文档构建无警告：`cargo doc --no-deps`

#### 部署注意事项

- 功能向后兼容，不影响现有用户
- 默认行为与 Python 版本一致

#### 需要更新的 CI 配置

- 无需更新 CI 配置

### 代码风格

遵循 AGENTS.md 规范：

#### 命名约定

- 变量和函数：`snake_case`
- 类型和 trait：`CamelCase`
- 测试函数：描述性名称，不使用 `test_` 前缀

#### 文档注释

使用 `///` 为公共 API 编写文档：

```rust
/// 追踪进度
///
/// # 参数
///
/// * `content` - 进度内容
/// * `is_tool_hint` - 是否为工具提示（true 表示工具调用提示，false 表示思考内容）
///
/// # 返回值
///
/// 成功返回 `Ok(())`，失败返回错误
///
/// # 示例
///
/// ```rust
/// use nanobot_agent::ProgressTracker;
/// use std::sync::Arc;
/// use anyhow::Result;
///
/// let tracker = Arc::new(|content: String, is_tool_hint: bool| {
///     Box::pin(async move {
///         println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
///         Ok(())
///     })
/// });
///
/// tracker.track("思考中...".to_string(), false).await?;
/// # Ok::<(), anyhow::Error>(())
/// ```
async fn track(&self, content: String, is_tool_hint: bool) -> Result<()>;
```

#### 格式化

提交前运行：
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

---

## 参考文档

- Python 版本实现：`_nanobot/nanobot/agent/loop.py`（第 160-175 行：`_strip_think` 和 `_tool_hint` 方法；第 200-205 行：进度通知调用）
- 项目开发规范：`AGENTS.md`
- 消息类型定义：`crates/channels/src/messages/mod.rs`（第 108-128 行：进度消息构造和判断方法）
- AgentLoop 实现：`crates/agent/src/loop/mod.rs`

---

## 时间估算

| 阶段 | 预计工作量 |
|------|-----------|
| 阶段 1：基础设施 | 4.9 小时 |
| 阶段 2：辅助方法 | 4.2 小时 |
| 阶段 3：核心集成 | 8.5 小时 |
| 阶段 4：测试与文档 | 10.5 小时 |
| **总计** | **28.1 小时** |

建议分配：单人开发 3-4 个工作日完成核心功能（阶段 1-3），第 5 天完成测试与文档（阶段 4）。
