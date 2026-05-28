# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/hook/mod.rs` | 新增 | Hook trait、HookCtx、CompositeHook、LoopHook |
| `crates/agent/src/hook/tests.rs` | 新增 | hook 模块单元测试 |
| `crates/agent/src/progress/mod.rs` | 删除 | 被 hook 模块取代 |
| `crates/agent/src/progress/tests.rs` | 删除 | 被 hook/tests.rs 取代 |
| `crates/agent/src/lib.rs` | 修改 | 删除 progress 模块，声明 hook 模块，更新公开导出 |
| `crates/agent/src/loop/mod.rs` | 修改 | re_act/process_direct/process_message/run 使用 hook |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | AgentCmd 构造 hook 替代 ProgressTracker |
| `crates/agent/AGENTS.md` | 修改 | 更新关键类型文档 |

## 任务列表

### 1. 新增 hook 模块：Hook trait + HookCtx + CompositeHook

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/hook/mod.rs`, `crates/agent/src/lib.rs`
- 验收标准: `cargo check -p nanobot-agent` 通过
- 风险/注意点: trait 方法签名需适配 Rust 类型系统
- 信心评估: 5
- 步骤:
  - [x] 创建 `crates/agent/src/hook/mod.rs`
  - [x] 定义 `HookCtx<'a>` 结构体（引用类型）：`content: &'a str`（当前迭代的 LLM 输出内容）, `tool_calls: &'a [ToolCall]`（当前迭代的工具调用列表）, `usage: Option<&'a Usage>`（token 用量）
  - [x] 定义 `Hook` trait（`#[async_trait]`），4 个方法全部提供默认空实现：`before_iteration(&self, ctx: &HookCtx<'_>)`、`before_execute_tools(&self, ctx: &HookCtx<'_>)`、`after_iteration(&self, ctx: &HookCtx<'_>)`、`finalize_content(&self, ctx: &HookCtx<'_>, content: Option<String>) -> Option<String>`
  - [x] 定义 `CompositeHook` 结构体（`hooks: Vec<Arc<dyn Hook>>`），实现 `Hook`：异步方法逐个调用并 `error!` 记录失败；`finalize_content` 串行管道传递
  - [x] 在 `lib.rs` 中声明 `mod hook;` 并 `pub use hook::{Hook, HookCtx, CompositeHook};`（暂不删除 progress 模块，等任务 3 一起处理）
  - [x] 运行 `cargo check -p nanobot-agent` 验证

### 2. 实现 LoopHook：替代 ChannelProgressTracker

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/hook/mod.rs`
- 验收标准: LoopHook 在 `before_execute_tools` 中发送思考内容和工具提示，行为与当前 ChannelProgressTracker 完全一致
- 风险/注意点: `strip_think` 和 `format_tool_hint` 函数在 `loop/mod.rs` 中是私有的，需改为 `pub(crate)` 或移到 hook 模块
- 信心评估: 5
- 步骤:
  - [x] 在 `hook/mod.rs` 中定义 `LoopHook` 结构体：`tx: mpsc::Sender<OutboundMessage>`, `channel: String`, `chat_id: String`
  - [x] 实现 `LoopHook::new(tx, channel, chat_id)` 构造方法
  - [x] 实现 `Hook` for `LoopHook`：`before_execute_tools` 中从 ctx 提取 content 和 tool_calls，调用 `strip_think` 发送思考内容（is_tool_hint=false），调用 `format_tool_hint` 发送工具提示（is_tool_hint=true）
  - [x] `finalize_content` 中调用 `strip_think` 剥离 `<think>` 标签
  - [x] 将 `strip_think` 和 `format_tool_hint` 从 `loop/mod.rs` 移到可被 hook 模块访问的位置（`pub(crate)` 或移到共享 utils）
  - [x] 添加 `pub use hook::LoopHook;` 到 `lib.rs`
  - [x] 运行 `cargo check -p nanobot-agent` 验证

### 3. 删除 progress 模块 + 重构 re_act 使用 hook

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/agent/src/lib.rs`, `crates/agent/src/progress/mod.rs`, `crates/agent/src/progress/tests.rs`
- 验收标准: progress 模块完全删除；re_act 签名改为 `hook: &dyn Hook`；`run()` 和 `process_message` 内部构造 LoopHook；所有现有测试通过
- 风险/注意点: `loop/tests.rs` 中可能有直接使用 ProgressTracker 的测试代码需要同步修改
- 信心评估: 4
- 步骤:
  - [x] 修改 `re_act` 签名：`on_progress: Option<Arc<dyn ProgressTracker>>` → `hook: &dyn Hook`
  - [x] 在 `re_act` 循环中插入 hook 调用点：迭代开始 `hook.before_iteration(&ctx)`，工具执行前 `hook.before_execute_tools(&ctx)`，工具执行后 `hook.after_iteration(&ctx)`
  - [x] 在最终返回前调用 `hook.finalize_content(&ctx, content)` 处理返回内容
  - [x] 删除 `re_act` 中原有的直接 ProgressTracker 调用代码（L243-258）
  - [x] 修改 `run()` 方法：将 `ChannelProgressTracker::new(...)` 替换为 `LoopHook::new(...)`
  - [x] 修改 `process_message` 内部的 `re_act` 调用，传入 hook 引用
  - [x] 修改 `process_direct` 签名：`on_progress: Option<Arc<dyn ProgressTracker>>` → `hook: Option<&dyn Hook>`；内部传给 `process_message`
  - [x] 提供一个 `struct NoopHook;` 实现（所有方法默认空操作），当没有 hook 时使用
  - [x] 从 `lib.rs` 中删除 `mod progress;` 和 `pub use progress::{...}`
  - [x] 删除 `crates/agent/src/progress/mod.rs` 和 `crates/agent/src/progress/tests.rs`
  - [x] 修改 `loop/tests.rs` 中使用 ProgressTracker 的测试代码
  - [x] 运行 `cargo test -p nanobot-agent` 验证所有测试通过
  - [x] 运行 `cargo clippy -p nanobot-agent -- -D warnings -D clippy::uninlined_format_args` 验证

### 4. 修改 nanobot binary crate 的调用方

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`
- 验收标准: AgentCmd 使用 Hook 替代 ProgressTracker；`cargo build` 通过
- 风险/注意点: CLI 模式的闭包回调需要改为实现 Hook trait 的结构体
- 信心评估: 4
- 步骤:
  - [x] 在 `commands/agent/mod.rs` 中定义 `CliHook` 结构体，持有 `send_tool_hints: bool`, `send_progress: bool`
  - [x] 实现 `Hook` for `CliHook`：`before_execute_tools` 中根据配置打印进度到 stdout；`finalize_content` 中调用 `strip_think`
  - [x] 将 `AgentCmd::run_single` 中的 `ProgressTracker` 闭包替换为 `CliHook` 实例
  - [x] 更新 import：`use nanobot_agent::ProgressTracker` → `use nanobot_agent::Hook`
  - [x] 运行 `cargo build -p nanobot` 验证编译通过
  - [x] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 验证

### 5. 新增 hook 模块单元测试

- 优先级: P1
- 依赖项: 3
- 涉及文件: `crates/agent/src/hook/tests.rs`
- 验收标准: 测试覆盖 CompositeHook 组合调用、错误隔离、finalize_content 管道、LoopHook 行为
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [x] 创建 `crates/agent/src/hook/tests.rs`，在 `hook/mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [x] 测试：默认 Hook 所有方法为空操作（不 panic）
  - [x] 测试：CompositeHook 按顺序调用所有 hook
  - [x] 测试：CompositeHook 中一个 hook 失败不影响其他 hook
  - [x] 测试：finalize_content 管道式传递（hook A 输出 → hook B 输入）
  - [x] 测试：LoopHook 在 before_execute_tools 中通过 tx 发送正确的 OutboundMessage
  - [x] 测试：LoopHook 的 finalize_content 剥离 `<think>` 标签
  - [x] 运行 `cargo test -p nanobot-agent` 验证

### 6. 更新 AGENTS.md 文档

- 优先级: P2
- 依赖项: 4
- 涉及文件: `crates/agent/AGENTS.md`
- 验收标准: 文档反映 Hook 取代 ProgressTracker
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [x] 删除 `ProgressTracker` 和 `ChannelProgressTracker` 的描述
  - [x] 添加 `Hook` trait、`HookCtx`、`CompositeHook`、`LoopHook` 说明
  - [x] 更新 `AgentLoop` 的方法签名描述（re_act、process_direct）
  - [x] 更新 Re-export 列表

## 实现建议

- `HookCtx<'a>` 使用引用类型（`&'a str`, `&'a [ToolCall]`, `Option<&'a Usage>`），在 re_act 循环中每次迭代构造，借用局部变量，零拷贝开销
- `strip_think` 和 `format_tool_hint` 移到 `crate::utils` 模块或标记为 `pub(crate)`，供 hook 和 loop 共用
- `NoopHook` 可以是一个空结构体 `pub struct NoopHook;`，利用 trait 默认方法实现零代码
- CLI 的 `CliHook` 放在 nanobot binary crate 中（不在 agent crate），因为它依赖 stdout 打印逻辑
