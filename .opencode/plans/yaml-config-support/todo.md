# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `Cargo.toml` | 修改 | 为 `config` crate 添加 `yaml` feature |
| `crates/config/Cargo.toml` | 修改 | 将 `serde_yaml` 从 dev-dependencies 移至 dependencies |
| `crates/config/src/lib.rs` | 修改 | 添加 `resolve_config_path()` 函数，更新 `CONFIG_PATH` 默认值 |
| `crates/config/src/schema/mod.rs` | 修改 | 更新 `load`/`load_from_path`/`save` 支持 YAML，添加 `ConfigError::Yaml` 变体，更新模块文档 |
| `crates/config/src/schema/tests.rs` | 修改 | 添加 YAML 加载、保存、环境变量覆盖的测试 |
| `crates/nanobot/src/commands/onboard/mod.rs` | 修改 | 更新提示信息，显示实际配置路径 |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | 更新提示信息，显示实际配置路径 |

## 任务列表

### 1. ✅ 添加 YAML 依赖

- 优先级: P0
- 依赖项: 无
- 涉及文件: `Cargo.toml`, `crates/config/Cargo.toml`
- 验收标准: `cargo check -p nanobot-config` 编译通过，`serde_yaml` 可在非 test 代码中使用
- 风险/注意点: 无
- 步骤:
  - [ ] 在 `Cargo.toml` 的 `[workspace.dependencies.config]` 中，将 `features` 从 `["json", "convert-case"]` 改为 `["json", "yaml", "convert-case"]`
  - [ ] 在 `crates/config/Cargo.toml` 中，将 `serde_yaml.workspace = true` 从 `[dev-dependencies]` 移至 `[dependencies]`
  - [ ] 运行 `cargo check -p nanobot-config` 确认编译通过

### 2. ✅ 添加 resolve_config_path 并更新 CONFIG_PATH

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/config/src/lib.rs`
- 验收标准: `resolve_config_path()` 可从 `nanobot_config` crate 公开访问；`CONFIG_PATH` 指向 `config.yaml`
- 风险/注意点: 无
- 步骤:
  - [ ] 添加公开函数 `resolve_config_path() -> Option<PathBuf>`：按 `config.json` > `config.yaml` > `config.yml` 顺序在 `NANOBOT_HOME_DIR` 下查找第一个存在的文件路径
  - [ ] 将 `CONFIG_PATH` 的默认值从 `config.json` 改为 `config.yaml`（新建配置的默认路径）
  - [ ] 导出 `resolve_config_path`

### 3. ✅ 更新 Config 加载逻辑支持 YAML

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/config/src/schema/mod.rs`
- 验收标准: `Config::load()` 能自动检测并加载 YAML 或 JSON 配置文件；`load_from_path` 能根据文件扩展名自动选择解析格式
- 风险/注意点: 无
- 步骤:
  - [ ] 修改 `load()` 方法：使用 `resolve_config_path()` 查找配置文件，存在则调用 `load_from_path`，不存在则返回 `Ok(None)`
  - [ ] 修改 `load_from_path`：将 `config::File::from(path).format(config::FileFormat::Json)` 改为 `config::File::from(path)`，让 `config` crate 根据文件扩展名自动选择解析格式
  - [ ] 运行 `cargo test -p nanobot-config` 确认现有 JSON 测试仍全部通过

### 4. ✅ 更新 Config 保存逻辑支持 YAML

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/config/src/schema/mod.rs`
- 验收标准: `save()` 能根据已有配置文件格式保存；无已有文件时默认保存为 YAML 格式到 `config.yaml`；YAML 文件权限为 600
- 风险/注意点: YAML 输出的字段名为 `camelCase`（由 `#[serde(rename_all = "camelCase")]` 控制）
- 步骤:
  - [ ] 在 `ConfigError` 枚举中添加 `Yaml(String)` 变体，错误消息格式为 `"YAML 序列化错误: {0}"`
  - [ ] 添加 `impl From<serde_yaml::Error> for ConfigError`，转换为 `ConfigError::Yaml(e.to_string())`
  - [ ] 修改 `save()` 方法：使用 `resolve_config_path()` 确定保存路径，未找到已有文件时回退到 `CONFIG_PATH`（即 `config.yaml`）
  - [ ] 根据保存路径的扩展名选择序列化方式：`"yaml" | "yml"` 使用 `serde_yaml::to_string`，其他使用 `serde_json::to_string_pretty`
  - [ ] 保留现有的目录创建、权限设置（0o600）和 `sync_all` 逻辑不变
  - [ ] 运行 `cargo test -p nanobot-config` 确认通过

### 5. ✅ 更新模块文档

- 优先级: P1
- 依赖项: 3, 4
- 涉及文件: `crates/config/src/schema/mod.rs`
- 验收标准: 模块文档准确描述 YAML 和 JSON 双格式支持，包含 YAML 配置示例；`cargo doc --no-deps -p nanobot-config` 无警告
- 风险/注意点: 无
- 步骤:
  - [ ] 更新模块顶部文档注释：将"采用 JSON 格式"改为"支持 JSON 和 YAML 格式"，说明支持 `config.json`、`config.yaml`、`config.yml`
  - [ ] 在现有 JSON 配置示例后添加等价的 YAML 配置示例
  - [ ] 更新 `load()` 方法的文档注释：说明支持 `config.json`、`config.yaml`、`config.yml`
  - [ ] 运行 `cargo doc --no-deps -p nanobot-config` 确认无警告

### 6. ✅ 添加 YAML 相关测试

- 优先级: P0
- 依赖项: 3, 4
- 涉及文件: `crates/config/src/schema/tests.rs`
- 验收标准: 所有新增测试通过；覆盖 YAML 加载、保存、环境变量覆盖等场景
- 风险/注意点: 测试中使用 `tempfile` 创建临时目录，文件名需使用 `.yaml`/`.yml` 扩展名
- 步骤:
  - [ ] 添加测试 `load_yaml_config`：创建 `.yaml` 临时文件写入 YAML 格式配置，调用 `Config::load_from_path` 验证反序列化正确
  - [ ] 添加测试 `load_yml_config`：同上但使用 `.yml` 扩展名
  - [ ] 添加测试 `yaml_env_override`：创建 YAML 配置文件，设置环境变量覆盖，验证环境变量优先级高于文件值
  - [ ] 添加测试 `save_yaml_format`：创建 `.yaml` 临时文件，加载后调用保存逻辑，验证输出文件内容为合法 YAML（以非 `{` 字符开头）
  - [ ] 添加测试 `save_json_format_when_json_exists`：创建 `.json` 临时文件，验证保存时仍使用 JSON 格式（以 `{` 开头）
  - [ ] 运行 `cargo test -p nanobot-config` 确认全部通过

### 7. ✅ 更新 onboard 命令的提示信息

- 优先级: P1
- 依赖项: 2
- 涉及文件: `crates/nanobot/src/commands/onboard/mod.rs`
- 验收标准: onboard 命令的提示信息显示实际配置文件路径（而非硬编码 `config.json`）
- 风险/注意点: 首次运行时 `resolve_config_path()` 在 `save()` 之前返回 `None`，应在 `save()` 之后再获取路径
- 步骤:
  - [ ] 将 `use nanobot_config::{CONFIG_PATH, Config}` 改为 `use nanobot_config::{Config, resolve_config_path, CONFIG_PATH}`
  - [ ] 将第 67 行硬编码的 `~/.nanobot/config.json` 改为使用 `resolve_config_path()` 获取实际路径，回退到 `CONFIG_PATH`
  - [ ] 运行 `cargo build -p nanobot` 确认编译通过

### 8. ✅ 更新 gateway 命令的提示信息

- 优先级: P1
- 依赖项: 2
- 涉及文件: `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: gateway 命令的提示信息显示实际配置文件路径
- 风险/注意点: 同任务 7
- 步骤:
  - [ ] 将第 174 行硬编码的 `~/.nanobot/config.json` 改为使用 `resolve_config_path()` 获取实际路径，回退到 `CONFIG_PATH`
  - [ ] 运行 `cargo build -p nanobot` 确认编译通过

### 9. ✅ 最终验证

- 优先级: P0
- 依赖项: 1-8
- 涉及文件: 无（全局验证）
- 验收标准: 所有检查命令通过，无警告无错误
- 风险/注意点: 无
- 步骤:
  - [ ] 运行 `cargo +nightly fmt` 格式化代码
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 确认无 lint 警告
  - [ ] 运行 `cargo test` 确认所有测试通过
  - [ ] 运行 `cargo doc --no-deps` 确认文档无警告

## 实现建议

- **加载侧**：`load_from_path` 使用 `config::File::from(path)` 替代 `config::File::from(path).format(config::FileFormat::Json)`，`config` crate 根据文件扩展名自动选择 JSON 或 YAML 解析器。`load()` 使用 `resolve_config_path()` 确定要加载的文件。
- **保存侧**：`config` crate 只负责加载不负责保存，`save()` 根据文件扩展名选择 `serde_json` 或 `serde_yaml` 进行序列化。
- 现有的 `#[serde(rename_all = "camelCase")]` 对 `serde_yaml` 同样生效，YAML 输出的字段名自动为 `camelCase`，无需额外配置。
- 现有测试中 `load_from_path` 使用 `.json` 扩展名的临时文件，`File::from(path)` 会自动识别为 JSON 格式，无需改动这些测试。
