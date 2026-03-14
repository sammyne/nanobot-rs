# 需求文档

## 引言

本功能旨在增强 nanobot-rs 项目的配置管理能力，通过引入 crates.io 上成熟的 `config` 库，实现环境变量对 `config.json` 配置项的覆盖能力。这将使应用在容器化部署、CI/CD 环境以及需要动态配置的场景下更加灵活，符合 12-Factor App 的配置管理原则。

当前配置系统仅支持从 `~/.nanobot/config.json` 文件加载配置，无法通过环境变量动态覆盖配置项，这在以下场景中存在限制：
- Docker/Kubernetes 部署时需要通过环境变量注入敏感信息（如 API Key）
- 不同环境（开发、测试、生产）需要不同的配置值
- CI/CD 流程中需要临时覆盖某些配置

## 需求

### 需求 1：引入 config 库并重构配置加载机制

**用户故事：** 作为开发者，我希望使用成熟的配置管理库来加载配置，以便获得更好的可维护性和扩展性。

#### 验收标准

1. WHEN 项目依赖中添加 `config` crate THEN 系统 SHALL 使用工作空间依赖管理规范进行声明
2. WHEN 配置加载逻辑被重构 THEN 系统 SHALL 保持现有的 `Config::load()` API 不变，确保向后兼容
3. WHEN 配置文件不存在 THEN 系统 SHALL 返回与现有行为一致的错误信息
4. WHEN 配置文件格式错误 THEN 系统 SHALL 返回清晰的错误提示，指明具体的解析错误位置

### 需求 2：支持环境变量覆盖配置项

**用户故事：** 作为运维人员，我希望通过环境变量覆盖配置文件中的值，以便在容器化环境中灵活配置应用。

#### 验收标准

1. WHEN 设置环境变量 `NANOBOT_PROVIDERS__CUSTOM__API_KEY` THEN 系统 SHALL 覆盖 `config.json` 中 `providers.custom.apiKey` 的值
2. WHEN 设置环境变量 `NANOBOT_AGENTS__DEFAULTS__MODEL` THEN 系统 SHALL 覆盖 `config.json` 中 `agents.defaults.model` 的值
3. WHEN 设置环境变量 `NANOBOT_GATEWAY__PORT` THEN 系统 SHALL 覆盖 `config.json` 中 `gateway.port` 的值
4. IF 环境变量值与配置文件值类型不匹配（如字符串转数字）THEN 系统 SHALL 返回类型转换错误
5. WHEN 环境变量未设置 THEN 系统 SHALL 使用配置文件中的值作为默认值

### 需求 3：定义环境变量命名规范

**用户故事：** 作为开发者，我希望有清晰的环境变量命名规范，以便正确设置和使用环境变量。

#### 验收标准

1. WHEN 定义环境变量命名规范 THEN 系统 SHALL 使用 `NANOBOT_` 作为统一前缀
2. WHEN 映射嵌套配置项 THEN 系统 SHALL 使用双下划线分隔层级（如 `NANOBOT_PROVIDERS__CUSTOM__API_KEY`）
3. WHEN 配置项名称为 camelCase THEN 系统 SHALL 在环境变量中转换为 SCREAMING_SNAKE_CASE（如 `apiKey` → `API_KEY`）
4. WHEN 文档化环境变量 THEN 系统 SHALL 在代码注释和 README 中提供完整的环境变量映射表

### 需求 4：保持错误处理一致性

**用户故事：** 作为开发者，我希望配置加载的错误处理与项目现有规范一致，以便提供统一的用户体验。

#### 验收标准

1. WHEN 配置加载失败 THEN 系统 SHALL 使用 `thiserror` 定义的 `ConfigError` 枚举返回错误
2. WHEN 环境变量解析失败 THEN 系统 SHALL 返回 `ConfigError::Parse` 错误，包含详细的错误信息
3. WHEN 配置验证失败 THEN 系统 SHALL 返回 `ConfigError::Validation` 错误，指明具体的验证失败原因
4. IF 新增错误类型需要 THEN 系统 SHALL 在 `ConfigError` 枚举中添加新的变体

### 需求 5：编写测试用例

**用户故事：** 作为开发者，我希望有完整的测试覆盖，以便确保配置加载功能的正确性和稳定性。

#### 验收标准

1. WHEN 编写单元测试 THEN 系统 SHALL 遵循项目测试规范，测试代码与源代码分离
2. WHEN 测试环境变量覆盖 THEN 系统 SHALL 验证各种类型（字符串、数字、布尔值）的覆盖行为
3. WHEN 测试配置合并 THEN 系统 SHALL 验证环境变量优先级高于配置文件
4. WHEN 测试边界情况 THEN 系统 SHALL 包括空值、特殊字符、类型不匹配等场景
