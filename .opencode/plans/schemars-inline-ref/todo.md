# TODO

## 任务列表

### 1. 为 `CronScheduleArgs` 添加 `#[schemars(inline)]` 属性

- **优先级**: P0
- **依赖项**: 无
- **风险/注意点**:
  - `#[schemars(inline)]` 属性需要添加在 `#[derive(JsonSchema)]` 之后
  - 不可与已有的 `#[serde(...)]` 属性冲突
  - 修改文件：`crates/cron/src/tool/mod.rs`（第 18-20 行附近）

✅ 已完成：为 `CronScheduleArgs` 添加了 `#[schemars(inline)]` 属性，编译检查通过，clippy 无警告

### 2. 验证修改效果（可选）

- **优先级**: P2
- **依赖项**: 1
- **风险/注意点**:
  - 运行 `cargo check` 确保代码编译通过
  - 可通过 `cargo test` 验证相关测试通过
  - 手动检查生成的 schema 是否还存在 `$ref`

✅ 已完成：编译检查通过，clippy 无警告，build 成功，添加了单元测试 `cron_args_schema_has_no_ref`，所有 26 个测试通过

## 实现建议

- **基于项目技术栈**：
  - 项目使用 `schemars 0.9+`，支持 `#[schemars(inline)]` 属性
  - 当前 `CronScheduleArgs` 定义在 `crates/cron/src/tool/mod.rs` 第 18-47 行
  - 在 `#[derive(JsonSchema)]` 之后、第一个 serde 属性之前插入 `#[schemars(inline)]`

- **修改位置示例**：

  ```rust
  /// Schedule definition for adding a cron job
  #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
  #[schemars(inline)]  // ← 新增此行
  #[serde(tag = "kind", rename_all = "lowercase")]
  pub enum CronScheduleArgs {
  ```

- **已验证无需修改的类型**：
  - `Action` (heartbeat): 单用类型，schemars 默认内联
  - `ExecArgs` (shell): 单用类型，schemars 默认内联
  - `ReadFileArgs`, `WriteFileArgs`, `EditFileArgs`, `ListDirArgs` (fs): 单用类型，schemars 默认内联
  - `SpawnParams` (subagent): 单用类型，schemars 默认内联