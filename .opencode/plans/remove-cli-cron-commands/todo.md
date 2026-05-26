# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/nanobot/src/commands/cron/mod.rs` | 修改 | 移除 List/Add/Remove 子命令及辅助函数，保留 Enable/Run |
| `crates/nanobot/src/commands/cron/tests.rs` | 修改 | 移除 AddCmd 和 format_schedule 相关测试 |
| `crates/nanobot/Cargo.toml` | 修改 | 移除 `nanobot-utils` 依赖 |
| `crates/nanobot/AGENTS.md` | 修改 | 更新 CronCmd 子命令描述 |

## 任务列表

### ✅ 1. 移除 List/Add/Remove 子命令及清理 cron/mod.rs

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/nanobot/src/commands/cron/mod.rs`
- 验收标准: 文件仅包含 `EnableCmd`、`RunCmd`、`format_time` 和对应的 `CronSubcommand` 枚举（Enable/Run 两个变体），编译通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 移除 `use nanobot_cron::CronSchedule;` import（第 6 行）
  - [ ] 从 `CronSubcommand` 枚举中移除 `List(ListCmd)`、`Add(AddCmd)`、`Remove(RemoveCmd)` 三个变体（第 35-39 行）
  - [ ] 从 `CronCmd::run()` 的 match 中移除 `List`、`Add`、`Remove` 三个分支（第 22-24 行）
  - [ ] 移除 `format_schedule` 函数（第 67-81 行）
  - [ ] 移除 `ListCmd` 结构体及其 impl 块（第 85-126 行）
  - [ ] 移除 `AddCmd` 结构体及其 impl 块（第 128-241 行）
  - [ ] 移除 `RemoveCmd` 结构体及其 impl 块（第 243-270 行）

### ✅ 2. 清理测试文件

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/nanobot/src/commands/cron/tests.rs`
- 验收标准: 仅保留 `test_format_time` 测试，`cargo test -p nanobot` 通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 移除 `test_format_schedule` 测试（第 17-33 行）
  - [ ] 移除 `test_build_schedule_every` 测试（第 35-48 行）
  - [ ] 移除 `test_build_schedule_cron` 测试（第 50-69 行）
  - [ ] 移除 `test_build_schedule_no_schedule` 测试（第 71-84 行）
  - [ ] 移除 `test_build_schedule_multiple_schedules` 测试（第 86-99 行）
  - [ ] 移除 `test_build_schedule_tz_without_cron` 测试（第 101-114 行）

### ✅ 3. 移除无用依赖

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/nanobot/Cargo.toml`
- 验收标准: `nanobot-utils` 不在 `[dependencies]` 中，`cargo check -p nanobot` 通过
- 风险/注意点: 已通过 grep 确认 `nanobot_utils` 在 nanobot crate 中仅被 `ListCmd` 使用（第 115 行），移除安全
- 信心评估: 5
- 步骤:
  - [ ] 从 `crates/nanobot/Cargo.toml` 的 `[dependencies]` 中移除 `nanobot-utils.workspace = true`（第 16 行）

### ✅ 4. 更新 crate 文档

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/nanobot/AGENTS.md`
- 验收标准: CronCmd 描述反映当前子命令（仅 enable/run）
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 将第 10 行 `子命令：\`list\`, \`add\`, \`remove\`, \`enable\`, \`run\`` 改为 `子命令：\`enable\`, \`run\``
  - [ ] 内部依赖列表中移除 `utils`（第 15 行）

### ✅ 5. 验证

- 优先级: P0
- 依赖项: 1, 2, 3, 4
- 涉及文件: 无
- 验收标准: 四项检查全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `cargo +nightly fmt`
  - [ ] `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`
  - [ ] `cargo test -p nanobot`
  - [ ] `cargo doc --no-deps`

## 实现建议

- 任务 1 中删除代码时从文件底部向上删，避免行号偏移影响定位
- `format_time` 保留原位不动，删除后它上方是 helper 注释区分线，下方是 EnableCmd，结构清晰
