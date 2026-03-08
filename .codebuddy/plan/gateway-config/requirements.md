# 需求文档

## 引言

本需求旨在为 Rust 版本的 nanobot 添加网关（Gateway）配置支持，使其与 Python 版本的配置结构保持一致。当前 Rust 版本的 `Config` 结构体缺少 `gateway` 配置字段，导致 gateway 命令无法从配置文件读取服务监听地址和端口，只能依赖命令行参数。本功能将补齐这一配置能力。

## 需求

### 需求 1：定义网关配置结构

**用户故事：** 作为 nanobot 的开发者，我希望在 Rust 配置模块中定义 `GatewayConfig` 结构体，以便能够从配置文件读取网关服务的监听参数。

#### 验收标准

1. WHEN 定义 `GatewayConfig` 结构体 THEN 系统 SHALL 包含 `host` 字段（类型为 `String`，默认值为 `"0.0.0.0"`）
2. WHEN 定义 `GatewayConfig` 结构体 THEN 系统 SHALL 包含 `port` 字段（类型为 `u16`，默认值为 `18790`）
3. WHEN 序列化/反序列化 `GatewayConfig` THEN 系统 SHALL 使用 camelCase 命名（即 JSON 字段名为 `host` 和 `port`）
4. IF `host` 字段未在配置文件中指定 THEN 系统 SHALL 使用默认值 `"0.0.0.0"`
5. IF `port` 字段未在配置文件中指定 THEN 系统 SHALL 使用默认值 `18790`

### 需求 2：集成网关配置到根配置

**用户故事：** 作为 nanobot 的用户，我希望在配置文件的顶层能够配置 gateway 字段，以便统一管理网关服务的参数。

#### 验收标准

1. WHEN 定义 `Config` 结构体 THEN 系统 SHALL 包含 `gateway` 字段（类型为 `GatewayConfig`）
2. IF 配置文件中未指定 `gateway` 字段 THEN 系统 SHALL 使用 `GatewayConfig::default()` 作为默认值
3. WHEN 保存配置文件 THEN 系统 SHALL 正确序列化 `gateway` 字段

### 需求 3：Gateway 命令使用配置文件参数

**用户故事：** 作为 nanobot 的用户，我希望 gateway 命令能够优先使用配置文件中的端口设置，同时保留命令行参数覆盖的能力，以便灵活配置服务端口。

#### 验收标准

1. WHEN 启动 gateway 命令且未指定 `--port` 参数 THEN 系统 SHALL 使用配置文件中 `gateway.port` 的值
2. WHEN 启动 gateway 命令且指定了 `--port` 参数 THEN 系统 SHALL 使用命令行参数的值（覆盖配置文件）
3. WHEN 配置文件中未指定 `gateway.port` THEN 系统 SHALL 使用默认端口 `18790`
4. WHEN 日志输出启动信息 THEN 系统 SHALL 显示实际使用的端口值及来源（配置文件或命令行）

### 需求 4：配置验证

**用户故事：** 作为 nanobot 的用户，我希望在加载配置时能够验证网关配置的有效性，以便在启动前发现配置错误。

#### 验收标准

1. WHEN `port` 值为 0 THEN 系统 SHALL 返回验证错误 "gateway.port 必须大于 0"
2. WHEN `host` 为空字符串 THEN 系统 SHALL 返回验证错误 "gateway.host 不能为空"
3. WHEN 配置验证通过 THEN 系统 SHALL 允许继续启动服务

### 需求 5：配置文件示例更新

**用户故事：** 作为 nanobot 的新用户，我希望配置文件示例中包含 gateway 配置的说明，以便了解如何正确配置网关参数。

#### 验收标准

1. WHEN 生成或展示配置文件示例 THEN 系统 SHALL 包含 `gateway` 配置节
2. WHEN 展示配置示例 THEN 系统 SHALL 显示 `host` 和 `port` 字段及其默认值
