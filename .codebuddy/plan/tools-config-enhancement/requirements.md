# 需求文档

## 引言

本需求文档描述对 `ToolsConfig` 配置结构的增强，旨在提供更细粒度的工具行为控制。当前 `ToolsConfig` 仅包含 `mcp_servers` 配置，需要扩展以支持全局工具限制策略和 `ExecTool` 的专属配置。

## 需求

### 需求 1：新增 restrict_to_workspace 全局字段

**用户故事：** 作为系统管理员，我希望能够配置工具是否限制在工作空间内执行，以便增强系统安全性。

#### 验收标准

1. WHEN `ToolsConfig` 被实例化 THEN 系统 SHALL 默认将 `restrict_to_workspace` 设置为 `false`
2. WHEN 用户在配置文件中指定 `restrictToWorkspace: true` THEN 系统 SHALL 限制所有文件系统相关工具仅在工作空间目录内操作
3. WHEN 配置文件未指定 `restrictToWorkspace` 字段 THEN 系统 SHALL 使用默认值 `false`，保持向后兼容

### 需求 2：新增 exec 字段支持 ExecTool 配置

**用户故事：** 作为开发者，我希望能够独立配置 ExecTool 的超时时间和 PATH 环境变量，以便灵活控制 Shell 命令执行行为。

#### 验收标准

1. WHEN `ToolsConfig` 被序列化或反序列化 THEN 系统 SHALL 支持 `exec` 字段，类型为 `ExecConfig`
2. WHEN 用户在配置文件中指定 `exec.timeout` THEN 系统 SHALL 在执行 Shell 命令时应用该超时时间（单位：秒）
3. WHEN 用户在配置文件中指定 `exec.pathAppend` THEN 系统 SHALL 将指定路径追加到命令执行时的 PATH 环境变量
4. WHEN 配置文件未指定 `exec` 字段或其子字段 THEN 系统 SHALL 使用默认值（`timeout: 60`，`pathAppend: ""`）
5. IF `exec.timeout` 被配置 THEN 该值 SHALL 覆盖 `ExecToolOptions` 的默认超时设置

### 需求 3：配置结构定义

**用户故事：** 作为系统维护者，我希望配置结构清晰且符合项目现有的 Serde 序列化规范，以便保持代码一致性。

#### 验收标准

1. WHEN 定义 `ExecConfig` 结构 THEN 系统 SHALL 使用 `#[serde(rename_all = "camelCase")]` 属性
2. WHEN 定义 `ToolsConfig` 新字段 THEN 系统 SHALL 使用 `#[serde(default)]` 属性确保向后兼容
3. WHEN `ExecConfig` 被定义 THEN 系统 SHALL 包含 `timeout: u64` 和 `path_append: String` 两个字段

### 需求 4：配置集成与应用

**用户故事：** 作为用户，我希望在配置文件中设置的值能够正确应用到实际工具执行中，以便配置生效。

#### 验收标准

1. WHEN `ToolRegistry` 创建 `ExecTool` THEN 系统 SHALL 读取 `ToolsConfig` 中的 `exec` 配置并应用到 `ExecToolOptions`
2. WHEN `ToolsConfig.restrict_to_workspace` 为 `true` THEN 系统 SHALL 将该值传递给 `ExecToolOptions.restrict_to_workspace`
3. WHEN 配置加载完成 THEN 系统 SHALL 能够正确解析包含新字段的 JSON 配置文件
