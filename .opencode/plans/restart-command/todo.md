# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/agent/src/cmd/restart/mod.rs` | 新增 | `RestartCmd` 实现 |
| `crates/agent/src/cmd/restart/tests.rs` | 新增 | 测试 |
| `crates/agent/src/cmd/mod.rs` | 修改 | 注册 `restart` 模块，导出 `RestartCmd` |
| `crates/agent/src/cmd/help/mod.rs` | 修改 | `/help` 输出包含 `/restart` |
| `crates/agent/src/loop/mod.rs` | 修改 | `try_handle_cmd` 中添加 `"restart"` 分支 |
| `crates/agent/src/loop/tests.rs` | 修改 | 适配 `/help` 输出变更 |

## 任务列表

### 1. 新增 RestartCmd

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/agent/src/cmd/restart/mod.rs`, `crates/agent/src/cmd/restart/tests.rs`, `crates/agent/src/cmd/mod.rs`
- 验收标准: `RestartCmd` 实现 `Command` trait；`run()` 返回 "Restarting..." 并 spawn 新进程后调用 `std::process::exit(0)`
- 信心评估: 4
- 步骤:
  - [ ] 创建 `crates/agent/src/cmd/restart/mod.rs`，定义 `RestartCmd` 结构体
  - [ ] 实现 `Command` trait 的 `run()` 方法：使用 `std::env::current_exe()` 获取当前可执行文件路径，`std::env::args().skip(1)` 获取参数，`std::process::Command::new(exe).args(args).spawn()` 启动新进程，然后 `std::process::exit(0)` 退出当前进程
  - [ ] 创建 `crates/agent/src/cmd/restart/tests.rs`，添加 `#[cfg(test)] mod tests;` 到 mod.rs
  - [ ] 在 `crates/agent/src/cmd/mod.rs` 中添加 `mod restart;` 和 `pub use restart::RestartCmd;`
  - [ ] 运行 `cargo check -p nanobot-agent`

### 2. 注册到 try_handle_cmd 和 /help

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/agent/src/cmd/help/mod.rs`, `crates/agent/src/loop/tests.rs`
- 验收标准: `/restart` 命令被识别并执行；`/help` 输出包含 `/restart` 说明
- 信心评估: 5
- 步骤:
  - [ ] 在 `try_handle_cmd` 的 match 中添加 `"restart"` 分支，创建 `RestartCmd` 并调用 `run()`
  - [ ] 在 `help/mod.rs` 的帮助文本中添加 `/restart — Restart the agent process`
  - [ ] 更新 `loop/tests.rs` 中所有 `/help` 预期输出字符串，包含 `/restart`
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过
