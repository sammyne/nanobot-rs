# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/cron/src/tool/mod.rs` | 修改 | CronArgs/CronArgsSchema 新增 deliver 字段，handle_add 接受 deliver |

## 任务列表

### 1. 暴露 deliver 参数给 LLM

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/cron/src/tool/mod.rs`
- 验收标准: LLM 可通过 `deliver: false` 创建静默任务；不传 deliver 时默认 true；现有测试通过
- 风险/注意点: 需要一个 `fn default_deliver() -> bool { true }` 辅助函数供 serde default 使用
- 信心评估: 5
- 步骤:
  - [ ] 添加 `fn default_deliver() -> bool { true }`
  - [ ] `CronArgs::Add` 新增 `#[serde(default = "default_deliver")] deliver: bool`
  - [ ] `CronArgsSchema` 新增 `#[serde(default = "default_deliver")] deliver: bool`，附带描述
  - [ ] `handle_add` 签名新增 `deliver: bool` 参数，替代硬编码 `true` 传入 `add_job`
  - [ ] `execute` match arm `CronArgs::Add { message, schedule, deliver }` 传递 deliver
  - [ ] 修复编译错误（测试中直接构造 CronArgs 的地方）
  - [ ] `cargo test -p nanobot-cron` 验证
  - [ ] `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 验证

## 实现建议

- `default_deliver` 函数放在 `CronArgsSchema` 定义之后、`CronTool` 定义之前
- 上游默认 `true` 是为了向后兼容（原来所有任务都发送通知），Rust 版应对齐
