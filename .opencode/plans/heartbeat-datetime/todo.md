# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/heartbeat/Cargo.toml` | 修改 | 新增 chrono 依赖 |
| `crates/heartbeat/src/service/mod.rs` | 修改 | `decide()` user message 注入当前时间 |

## 任务列表

### 1. heartbeat decide() 注入当前时间

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/heartbeat/Cargo.toml`, `crates/heartbeat/src/service/mod.rs`
- 验收标准: heartbeat Phase 1 的 user message 包含 `Current Time: ...` 行
- 信心评估: 5
- 步骤:
  - [ ] `crates/heartbeat/Cargo.toml` 新增 `chrono.workspace = true`
  - [ ] `decide()` 中构造 user message 时，用 `chrono::Local::now()` 格式化时间，追加到 prompt 开头
  - [ ] `cargo check -p nanobot-heartbeat` 确认编译通过
