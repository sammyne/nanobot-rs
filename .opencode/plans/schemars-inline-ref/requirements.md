# 需求

## 目标与背景

当前项目使用 `schemars` crate 的 `#[derive(JsonSchema)]` 宏生成 JSON Schema，用于 MCP 工具参数定义。生成的 schema 中存在 `$ref` 字段引用内嵌结构体（如 `CronScheduleArgs`），这会导致某些 JSON Schema 处理工具无法正确解析完整 schema 结构。

目标是让所有嵌套结构体的 schema 定义直接内联到父 schema 中，而非通过 `$ref` 引用。

## 功能需求列表

### 核心功能

- **修改 `CronScheduleArgs` 类型定义**
  - 在 `crates/cron/src/tool/mod.rs` 文件中
  - 为 `CronScheduleArgs` enum 添加 `#[schemars(inline)]` 属性
  - 预期效果：`CronArgs` 的 schema 中不再出现 `$ref` 指向 `CronScheduleArgs`，而是直接内联其定义

### 扩展功能

- **验证其他类型的 `$ref` 存在性**（可选）
  - 检查其他 `#[derive(JsonSchema)]` 的类型是否也存在不必要的 `$ref`
  - 确认以下类型无需修改（已验证为单用类型，schemars 默认内联）：
    - `Action` (heartbeat/src/service.rs)
    - `ExecArgs` (tools/src/shell/mod.rs)
    - `ReadFileArgs`, `WriteFileArgs`, `EditFileArgs`, `ListDirArgs` (tools/src/fs.rs)
    - `SpawnParams` (subagent/src/tool.rs)

## 非功能需求

- **性能**：schema 生成使用 `LazyLock`，修改后无性能影响
- **安全**：仅修改属性宏，不涉及业务逻辑
- **兼容性**：修改后的 schema 仍需兼容 JSON Schema 规范（内联而非引用）
- **可维护性**：单一修改点，集中管理

## 边界与不做事项

- **不做**：
  - 不修改任何业务逻辑代码
  - 不修改 serde 属性（如 `#[serde(tag = "kind")]`)
  - 不修改其他不产生 `$ref` 的类型
  - 不修改 schema 生成方式（仍使用 `schemars::schema_for!` + `LazyLock`）

## 假设与约束

- **技术假设**：
  - 项目使用 schemars 0.9+ 版本（支持 `#[schemars(inline)]` 属性）
  - `#[schemars(inline)]` 宏只影响 schema 生成，不影响 serde 序列化
- **资源约束**：单次提交即可完成
- **环境约束**：无 CI/CD 特殊要求