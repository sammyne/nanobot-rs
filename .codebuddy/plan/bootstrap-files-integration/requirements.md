# 需求文档

## 引言
本功能旨在将 Python 版本 nanobot 中的 bootstrap 文件集成机制移植到 Rust 版本的上下文构建组件中。Bootstrap 文件包含关键的配置、身份和工具信息，用于为 AI 代理提供系统级上下文和指导。这些文件使代理能够理解其角色、行为准则、可用工具和用户偏好。

## 需求

### 需求 1

**用户故事：** 作为【开发者】，我希望【Rust 版本的 ContextBuilder 能够加载工作区中的 bootstrap 文件】，以便【构建与 Python 版本一致的系统提示词，确保代理行为的兼容性】。

#### 验收标准

1. WHEN 【系统初始化 ContextBuilder】 THEN 【系统】 SHALL 【定义 bootstrap 文件列表（AGENTS.md、SOUL.md、USER.md、TOOLS.md、IDENTITY.md）】
2. WHEN 【构建系统提示词】 AND 【工作区存在 bootstrap 文件】 THEN 【系统】 SHALL 【通过 `load_bootstrap_files` 方法加载文件内容并将其包含在系统提示词中】
3. WHEN 【构建系统提示词】 AND 【某个 bootstrap 文件不存在】 THEN 【系统】 SHALL 【跳过该文件而不报错，继续处理其他文件】
4. WHEN 【bootstrap 文件被加载】 THEN 【系统】 SHALL 【使用文件名作为章节标题（如 "## AGENTS.md"）】
5. WHEN 【 bootstrap 文件内容为空或仅包含空白字符】 THEN 【系统】 SHALL 【不添加该文件的章节到系统提示词中】
6. WHEN 【至少存在一个有效的 bootstrap 文件】 THEN 【系统】 SHALL 【将所有加载的文件内容用换行符连接】
7. WHEN 【不存在任何有效的 bootstrap 文件】 THEN 【系统】 SHALL 【返回空字符串】

### 需求 2

**用户故事：** 作为【开发者】，我希望【bootstrap 文件内容被插入到系统提示词的适当位置】，以便【确保信息展示的逻辑顺序（核心身份 → bootstrap 文件 → 内存 → 技能）】。

#### 验收标准

1. WHEN 【构建系统提示词】 THEN 【系统】 SHALL 【按顺序组装以下部分：核心身份、bootstrap 文件、内存上下文、活跃技能、技能摘要】
2. WHEN 【组装各部分内容】 THEN 【系统】 SHALL 【使用 "---\n\n" 作为各部分之间的分隔符】
3. IF 【 bootstrap 文件部分为空】 THEN 【系统】 SHALL 【不添加分隔符，直接连接核心身份和内存上下文】
4. WHEN 【系统提示词构建完成】 THEN 【系统】 SHALL 【确保 bootstrap 文件内容位于核心身份之后、内存上下文之前】

### 需求 3

**用户故事：** 作为【开发者】，我希望【文件加载过程能够正确处理文件读取错误】，以便【避免因单个文件错误导致整个上下文构建失败】。

#### 验收标准

1. WHEN 【读取 bootstrap 文件时发生 IO 错误】 THEN 【系统】 SHALL 【记录警告日志并跳过该文件】
2. WHEN 【bootstrap 文件编码不是 UTF-8】 THEN 【系统】 SHALL 【记录警告日志并跳过该文件】
3. WHEN 【加载 bootstrap 文件】 THEN 【系统】 SHALL 【不影响系统提示词的生成流程】

### 需求 4

**用户故事：** 作为【开发者】，我希望【代码遵循 Rust 最佳实践和项目规范】，以便【保持代码质量和可维护性】。

#### 验收标准

1. WHEN 【实现 bootstrap 文件加载功能】 THEN 【系统】 SHALL 【使用 Rust 标准库的文件系统 API（std::fs）】
2. WHEN 【处理文件路径】 THEN 【系统】 SHALL 【使用 PathBuf 类型确保跨平台兼容性】
3. WHEN 【记录日志】 THEN 【系统】 SHALL 【使用 tracing crate 进行结构化日志记录】
4. WHEN 【添加新方法】 THEN 【系统】 SHALL 【包含完整的文档注释和参数说明】
5. WHEN 【实现功能】 THEN 【系统】 SHALL 【编写单元测试覆盖正常和异常情况】