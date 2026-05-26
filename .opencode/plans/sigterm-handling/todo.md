# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | 用 `tokio::select!` 同时监听 ctrl_c 和 SIGTERM |

## 任务列表

### ✅ 1. 添加 SIGTERM 信号监听

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: gateway 同时响应 SIGINT 和 SIGTERM，触发优雅关闭
- 风险/注意点: `tokio::signal::unix` 仅在 Unix 平台可用，需 `#[cfg(unix)]` 条件编译
- 信心评估: 5
- 步骤:
  - [ ] 将 `tokio::signal::ctrl_c().await` 替换为 `tokio::select!`，同时等待 ctrl_c 和 SIGTERM
  - [ ] Unix 平台：`tokio::signal::unix::signal(SignalKind::terminate())` 监听 SIGTERM
  - [ ] 更新提示文本：`按 Ctrl+C 停止` → `按 Ctrl+C 或发送 SIGTERM 停止`

### ✅ 2. 验证

- 优先级: P0
- 依赖项: 1
- 涉及文件: 无
- 验收标准: 四项检查全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `cargo +nightly fmt`
  - [ ] `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`
  - [ ] `cargo test -p nanobot`
  - [ ] `cargo doc --no-deps`
