# 实施计划

- [ ] 1. 创建独立的 cron crate 结构并配置 workspace
   - 在 `crates/cron/` 目录下创建 Rust crate 结构（`Cargo.toml`、`src/lib.rs`）
   - 在根 `Cargo.toml` 的 `workspace.members` 中添加 `crates/cron`
   - 在 `workspace.dependencies` 中添加 `nanobot-cron.path = "crates/cron"`
   - 配置 `Cargo.toml` 使用 workspace 依赖模式
   - _需求：1.1、1.2、1.3、7.1、7.2、7.3_

- [ ] 2. 迁移 cron 核心类型定义
   - 将 `crates/tools/src/cron_types.rs` 迁移至 `crates/cron/src/types.rs`
   - 保持 `CronSchedule`、`CronJob`、`CronJobState`、`CronPayload`、`CronStore` 的公共 API 不变
   - 保持 `Serialize`、`Deserialize` trait 实现
   - 在 `lib.rs` 中公开类型定义
   - _需求：2.1、2.2、2.3_

- [ ] 3. 迁移 cron 调度器模块
   - 将 `crates/tools/src/cron/scheduler.rs` 迁移至 `crates/cron/src/scheduler.rs`
   - 保持 `compute_next_run`、`validate_schedule`、`is_valid_timezone` 函数签名不变
   - 在 `Cargo.toml` 中声明 `chrono`、`chrono-tz`、`cron` 依赖
   - 在 `lib.rs` 中导出调度器模块
   - _需求：3.1、3.2、3.3_

- [ ] 4. 迁移 cron 存储模块
   - 将 `crates/tools/src/cron/storage.rs` 迁移至 `crates/cron/src/storage.rs`
   - 保持 `CronStorage` 的公共方法签名不变
   - 更新引用以使用同 crate 的 types 模块
   - 在 `lib.rs` 中导出存储模块
   - _需求：4.1、4.2、4.3_

- [ ] 5. 迁移 cron 服务模块
   - 将 `crates/tools/src/cron/service.rs` 迁移至 `crates/cron/src/service.rs`
   - 保持 `CronService` 的公共 API 不变
   - 更新引用以使用同 crate 的 scheduler、storage、types 模块
   - 在 `lib.rs` 中导出服务模块
   - _需求：5.1、5.2、5.3_

- [ ] 6. 迁移 CronTool 实现
   - 将 `CronTool` 定义从 `crates/tools/src/cron/mod.rs` 迁移至 `crates/cron/src/tool.rs`
   - 在 `Cargo.toml` 中添加对 `nanobot-tools` crate 的依赖，复用其 `Tool` trait
   - 在 `lib.rs` 中公开 `CronTool` 类型
   - _需求：6.1、6.2、6.3、6.4_

- [ ] 7. 迁移测试代码
   - 将各模块的 `tests.rs` 文件迁移到 `crates/cron/src/` 对应位置
   - 保持测试代码与源代码的 `#[cfg(test)] mod tests;` 结构一致
   - 运行 `cargo test -p nanobot-cron` 验证测试通过
   - _需求：8.1、8.2、8.3_

- [ ] 8. 从 tools crate 中移除已迁移的代码
   - 删除 `crates/tools/src/cron/` 目录
   - 删除 `crates/tools/src/cron_types.rs` 文件
   - 更新 `crates/tools/src/lib.rs` 移除相关模块引用
   - _需求：1.1_

- [ ] 9. 代码质量保证
   - 运行 `cargo fmt` 格式化代码
   - 运行 `cargo clippy --all-targets --all-features -- -D warnings -D clippy::uninlined_format_args` 检查
   - 修复所有 clippy 警告
   - 运行完整测试套件确保功能正确
   - _需求：9.1、9.2、9.3_
