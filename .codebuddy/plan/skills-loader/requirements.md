# 需求文档：Rust 版 Skills Loader 组件

## 引言

本组件是 Python 版 `SkillsLoader` 的 Rust 实现，用于管理 Agent 的技能（Skills）。技能是以 `SKILL.md` 文件形式存在的 Markdown 文档，包含 YAML 前置数据（frontmatter）作为元数据。该组件负责从工作空间和内置目录加载技能、解析元数据、检查依赖可用性，并为 Agent 上下文提供技能摘要和内容。

## 需求

### 需求 1：技能发现与加载

**用户故事：** 作为 Agent 系统，我希望能够发现和加载工作空间及内置目录中的技能，以便 Agent 能够获取可用的能力描述。

#### 验收标准

1. WHEN 初始化 SkillsLoader 时 THEN 系统 SHALL 接收工作空间路径和可选的内置技能目录路径
2. WHEN 调用技能列表功能时 THEN 系统 SHALL 扫描工作空间 `skills/` 目录和内置技能目录
3. IF 工作空间和内置目录存在同名技能 THEN 系统 SHALL 优先返回工作空间版本
4. WHEN 扫描技能目录时 THEN 系统 SHALL 仅识别包含 `SKILL.md` 文件的子目录
5. WHEN 加载技能内容时 THEN 系统 SHALL 返回技能文件的完整文本内容

### 需求 2：元数据解析

**用户故事：** 作为 Agent 系统，我希望能够解析技能文件的元数据，以便了解技能的描述、依赖和配置信息。

#### 验收标准

1. WHEN 技能文件包含 YAML 前置数据时 THEN 系统 SHALL 解析 `---` 包围的 YAML 块
2. WHEN 解析前置数据时 THEN 系统 SHALL 提取 `description`、`always` 等标准字段
3. WHEN 前置数据包含 `metadata` 字段时 THEN 系统 SHALL 将其作为 JSON 解析，并支持 `nanobot` 和 `openclaw` 两个键名
4. IF 元数据解析失败 THEN 系统 SHALL 返回空元数据而非错误

### 需求 3：依赖检查

**用户故事：** 作为 Agent 系统，我希望能够检查技能的依赖是否满足，以便只向用户提供可用的技能。

#### 验收标准

1. WHEN 检查技能依赖时 THEN 系统 SHALL 读取元数据中的 `requires` 字段
2. IF `requires.bins` 包含 CLI 工具名 THEN 系统 SHALL 检查该工具是否在 PATH 中可用
3. IF `requires.env` 包含环境变量名 THEN 系统 SHALL 检查该环境变量是否已设置
4. WHEN 列出技能时 THEN 系统 SHALL 支持过滤掉不满足依赖的技能
5. WHEN 技能不可用时 THEN 系统 SHALL 提供缺失依赖的描述信息

### 需求 4：技能摘要构建

**用户故事：** 作为 Agent 系统，我希望获得格式化的技能摘要，以便在上下文中向 Agent 展示可用技能列表。

#### 验收标准

1. WHEN 构建技能摘要时 THEN 系统 SHALL 生成 XML 格式的输出
2. WHEN 生成摘要时 THEN 系统 SHALL 包含技能名称、描述、路径位置
3. WHEN 生成摘要时 THEN 系统 SHALL 标注每个技能的可用状态（available 属性）
4. IF 技能不可用 THEN 系统 SHALL 在摘要中包含缺失的依赖信息

### 需求 5：上下文技能加载

**用户故事：** 作为 Agent 系统，我希望能够加载特定技能的内容，以便将其注入到 Agent 的上下文中。

#### 验收标准

1. WHEN 加载技能用于上下文时 THEN 系统 SHALL 移除 YAML 前置数据，仅保留 Markdown 正文
2. WHEN 加载多个技能时 THEN 系统 SHALL 使用分隔符格式化输出
3. WHEN 加载技能时 THEN 系统 SHALL 为每个技能添加名称标题

### 需求 6：Always 技能识别

**用户故事：** 作为 Agent 系统，我希望能够识别标记为 `always=true` 的技能，以便自动将其加载到上下文中。

#### 验收标准

1. WHEN 获取 always 技能列表时 THEN 系统 SHALL 筛选元数据中 `always` 字段为 true 的技能
2. WHEN 返回 always 技能时 THEN 系统 SHALL 仅返回满足依赖要求的技能

## 技术约束

1. 使用 `anyhow` 进行错误处理（应用程序类型）
2. 单元测试与源代码分离，测试函数命名不带 `test_` 前缀
3. 作为 Workspace 成员 crate，放入 `crates/` 目录
4. 使用 `serde` 进行 YAML/JSON 解析
