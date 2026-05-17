# 需求

## 目标与背景

当前 nanobot 仅支持 JSON 格式的配置文件（`~/.nanobot/config.json`）。需要新增 YAML 格式支持，使用户可以选择使用 `config.yaml` 或 `config.yml` 作为配置文件。YAML 格式相比 JSON 更适合手动编辑（支持注释、更简洁的语法），在 DevOps 和云原生场景中也更常见。

## 方案比较

### 方案 1: 多格式自动检测（推荐）

- 思路: 按优先级顺序查找配置文件（`config.yaml` > `config.yml` > `config.json`），加载找到的第一个文件。`save()` 方法根据当前加载的文件格式保存，新建配置默认使用 YAML 格式。
- 优点: 用户无需额外配置，放置对应格式的文件即可；向后兼容现有 JSON 配置；支持用户自由选择偏好格式。
- 缺点: 需要处理多个文件同时存在的优先级逻辑；`save()` 需要记住当前格式。

### 方案 2: 仅新增 YAML 支持，保留 JSON 默认

- 思路: 与方案 1 类似的自动检测，但优先级为 `config.json` > `config.yaml` > `config.yml`，新建配置仍默认使用 JSON 格式。
- 优点: 对现有用户完全无感知；最小化变更风险。
- 缺点: 新用户仍然默认得到 JSON 配置，无法体现 YAML 的优势。

### 方案 3: 通过命令行参数指定格式

- 思路: 添加 `--config-format yaml|json` 参数，显式指定配置文件格式。
- 优点: 完全明确，无歧义。
- 缺点: 增加用户使用复杂度；每次运行都需要指定；不符合"约定优于配置"的原则。

### 推荐

采用方案 2 的优先级（`config.json` > `config.yaml` > `config.yml`），结合方案 1 的自动检测机制。加载时由 `resolve_config_path()` 按固定顺序查找，`config` crate 的 `File::from(path)` 根据扩展名自动选择解析格式。新建配置默认使用 YAML 格式。

## 功能需求列表

### 核心功能

- 支持从 `~/.nanobot/config.yaml` 或 `~/.nanobot/config.yml` 加载 YAML 格式配置
- 配置文件查找优先级：`config.json` > `config.yaml` > `config.yml`
- 根据实际加载的文件格式决定 `save()` 的序列化格式
- 新建配置（`onboard` 命令）默认生成 YAML 格式
- YAML 和 JSON 配置统一使用 `camelCase` 字段命名（如 `apiKey`、`maxTokens`），复用现有 `#[serde(rename_all = "camelCase")]`
- 环境变量覆盖机制在两种格式下均正常工作

### 扩展功能

- 无（保持简单）

## 非功能需求

- **性能**：配置文件通常很小（< 1KB），格式检测的性能开销可忽略
- **安全**：YAML 配置文件与 JSON 一样设置 600 权限（仅当前用户可读写）
- **兼容性**：现有 JSON 配置文件无需任何修改即可继续使用
- **可维护性**：加载侧利用 `config` crate 的 `File::from(path)` 根据扩展名自动选择解析格式；保存侧根据文件扩展名选择序列化器
- **测试要求**：为 YAML 加载、保存、格式检测、环境变量覆盖等场景编写单元测试；测试代码与源代码分离到 `tests.rs`

## 边界与不做事项

- 不支持 TOML 或其他格式（仅 JSON 和 YAML）
- 不支持同时加载多个配置文件合并
- 不提供 JSON 到 YAML 的迁移工具
- 不修改配置的 schema 结构（仅改变序列化格式）

## 假设与约束

- **技术假设**：`config` crate（crates.io）的 `yaml` feature 可正常工作；`serde_yaml` 0.9 与现有 serde 派生兼容
- **资源约束**：无特殊约束
- **环境约束**：Rust >= 1.93，与现有 CI/CD 流程兼容

## 待确认事项

- 无
