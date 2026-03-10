# 需求文档：将 Cron 工具独立为独立 Crate

## 引言

当前 cron 相关功能代码位于 `crates/tools` crate 中，包括调度器、服务层、存储层和类型定义。为了提高代码的可维护性和复用性，需要将 cron 功能独立为一个单独的 `cron` crate。

## 当前状态

### 文件结构
```
crates/tools/
├── src/
│   ├── core.rs              # Tool trait 定义（需要保留）
│   ├── cron/                # 需要迁移的 cron 模块
│   │   ├── mod.rs           # CronTool 定义
│   │   ├── scheduler.rs     # 调度计算逻辑
│   │   ├── service.rs       # CronService 服务
│   │   └── storage.rs       # 存储层
│   ├── cron_types.rs        # 需要迁移的类型定义
│   ├── fs.rs                # 文件系统工具（保留）
│   ├── registry.rs          # 工具注册表（保留）
│   └── shell.rs             # Shell 工具（保留）
```

### 依赖关系
- `CronTool` 依赖 `Tool` trait（定义在 `core.rs`）
- cron 模块使用的外部依赖：`chrono`, `chrono-tz`, `cron`, `uuid`, `tokio`, `serde`, `serde_json`, `schemars`, `async-trait`, `thiserror`, `anyhow`, `tracing`

## 需求

### 需求 1：创建独立的 cron crate 结构

**用户故事：** 作为开发者，我希望 cron 功能有独立的 crate 结构，以便于独立维护和复用。

#### 验收标准

1. WHEN 创建新 crate THEN 系统 SHALL 在 `crates/cron/` 目录下创建完整的 Rust crate 结构
2. WHEN 配置 crate THEN 系统 SHALL 包含 `Cargo.toml`、`src/lib.rs` 及相应的模块文件
3. WHEN 配置依赖 THEN 系统 SHALL 使用 workspace 依赖模式（`workspace = true`）

### 需求 2：迁移 cron 核心类型定义

**用户故事：** 作为开发者，我希望 cron 相关类型独立定义，以便其他模块可以直接使用而不依赖 tools crate。

#### 验收标准

1. WHEN 迁移类型定义 THEN 系统 SHALL 将 `cron_types.rs` 迁移至 `crates/cron/src/types.rs`
2. WHEN 定义类型 THEN 系统 SHALL 保持 `CronSchedule`、`CronJob`、`CronJobState`、`CronPayload`、`CronStore` 的公共 API 不变
3. IF 类型需要序列化支持 THEN 系统 SHALL 保持 `Serialize`、`Deserialize` trait 实现

### 需求 3：迁移 cron 调度器模块

**用户故事：** 作为开发者，我希望调度计算逻辑独立，以便于单独测试和复用。

#### 验收标准

1. WHEN 迁移调度器 THEN 系统 SHALL 将 `scheduler.rs` 迁移至 `crates/cron/src/scheduler.rs`
2. WHEN 迁移后 THEN 系统 SHALL 保持 `compute_next_run`、`validate_schedule`、`is_valid_timezone` 函数签名不变
3. WHEN 配置依赖 THEN 系统 SHALL 在 `Cargo.toml` 中声明 `chrono`、`chrono-tz`、`cron` 依赖

### 需求 4：迁移 cron 存储模块

**用户故事：** 作为开发者，我希望存储层独立，以便于在其他项目中复用持久化逻辑。

#### 验收标准

1. WHEN 迁移存储模块 THEN 系统 SHALL 将 `storage.rs` 迁移至 `crates/cron/src/storage.rs`
2. WHEN 迁移后 THEN 系统 SHALL 保持 `CronStorage` 的公共方法签名不变
3. IF 存储模块依赖类型 THEN 系统 SHALL 正确引用同 crate 的 types 模块

### 需求 5：迁移 cron 服务模块

**用户故事：** 作为开发者，我希望服务层独立，以便于作为独立服务启动和管理。

#### 验收标准

1. WHEN 迁移服务模块 THEN 系统 SHALL 将 `service.rs` 迁移至 `crates/cron/src/service.rs`
2. WHEN 迁移后 THEN 系统 SHALL 保持 `CronService` 的公共 API 不变
3. IF 服务模块依赖其他模块 THEN 系统 SHALL 正确引用同 crate 的 scheduler、storage、types 模块

### 需求 6：迁移 CronTool 实现

**用户故事：** 作为开发者，我希望 CronTool 实现位于 cron crate 中，但保持与 Tool trait 的兼容性。

#### 验收标准

1. WHEN 迁移 CronTool THEN 系统 SHALL 将 `CronTool` 定义迁移至 `crates/cron/src/tool.rs`
2. WHEN 定义 CronTool THEN 系统 SHALL 保持对 `Tool` trait 的实现
3. IF `Tool` trait 定义在 tools crate THEN 系统 SHALL 添加对 `nanobot-tools` 的依赖或定义本地 trait 抽象
4. WHEN 导出 THEN 系统 SHALL 在 `lib.rs` 中公开 `CronTool` 类型

### 需求 7：更新 workspace 配置

**用户故事：** 作为开发者，我希望 workspace 正确配置新 crate，以便统一构建和依赖管理。

#### 验收标准

1. WHEN 添加新 crate THEN 系统 SHALL 在根 `Cargo.toml` 的 `workspace.members` 中包含 `crates/cron`
2. WHEN 配置路径依赖 THEN 系统 SHALL 在 `workspace.dependencies` 中添加 `nanobot-cron.path = "crates/cron"`
3. WHEN 其他 crate 需要使用 cron THEN 系统 SHALL 可以通过 `nanobot-cron.workspace = true` 引用

### 需求 8：迁移测试代码

**用户故事：** 作为开发者，我希望测试代码与源代码一起迁移，以便保证功能正确性。

#### 验收标准

1. WHEN 迁移测试 THEN 系统 SHALL 将各模块的 `tests.rs` 文件迁移到对应位置
2. WHEN 迁移后 THEN 系统 SHALL 保持测试代码与源代码的 `#[cfg(test)] mod tests;` 结构一致
3. WHEN 运行测试 THEN 系统 SHALL 通过 `cargo test -p nanobot-cron` 执行所有测试

### 需求 9：代码质量保证

**用户故事：** 作为开发者，我希望迁移后的代码通过 clippy 检查，以便保持代码质量。

#### 验收标准

1. WHEN 迁移完成后 THEN 系统 SHALL 通过 `cargo clippy --all-targets --all-features -- -D warnings -D clippy::uninlined_format_args` 检查
2. WHEN 格式化 THEN 系统 SHALL 通过 `cargo fmt --check` 检查
3. IF 存在 clippy 警告 THEN 系统 SHALL 修复所有警告
