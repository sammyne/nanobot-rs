# 需求文档

## 引言

本需求文档描述了为 Rust 版 nanobot CLI 的 `onboard` 命令补全工作空间模板实例化逻辑的功能。参照 Python 版的实现，在用户执行 `onboard` 命令时，需要自动创建工作空间目录和初始化模板文件，包括用户配置、Agent 指令、人格设定、工具说明、记忆存储等核心文件。

## 需求

### 需求 1：模板文件嵌入与资源管理

**用户故事：** 作为开发者，我希望模板文件能够嵌入到 Rust 二进制文件中，以便在不依赖外部文件系统的情况下正确分发和部署。

#### 验收标准

1. WHEN 编译 CLI 程序 THEN 系统 SHALL 使用 `include_dir` crate 将模板文件嵌入到二进制文件中
2. IF 模板目录存在 THEN 系统 SHALL 在编译时自动包含所有 `.md` 模板文件
3. WHEN 程序运行时需要访问模板内容 THEN 系统 SHALL 能够从嵌入的资源中读取模板内容

### 需求 2：工作空间目录创建

**用户故事：** 作为用户，我希望在执行 onboard 命令时自动创建工作空间目录结构，以便后续可以直接使用 AI Agent 功能。

#### 验收标准

1. WHEN 用户执行 onboard 命令 THEN 系统 SHALL 从配置中获取工作空间路径（`agents.defaults.workspace`）
2. IF 工作空间目录不存在 THEN 系统 SHALL 创建完整的工作空间目录路径
3. WHEN 目录创建成功 THEN 系统 SHALL 在控制台显示成功消息

### 需求 3：根级别模板文件创建

**用户故事：** 作为用户，我希望工作空间中包含预设的配置模板文件，以便快速开始使用并自定义 AI Agent 行为。

#### 验收标准

1. WHEN 工作空间创建完成 THEN 系统 SHALL 在根目录创建以下模板文件（如不存在）：
   - `USER.md` - 用户配置文件
   - `AGENTS.md` - Agent 指令文件
   - `SOUL.md` - AI 人格设定文件
   - `TOOLS.md` - 工具使用说明文件
2. IF 目标文件已存在 THEN 系统 SHALL 跳过该文件的创建（不覆盖用户已有配置）
3. WHEN 每个文件创建成功 THEN 系统 SHALL 在控制台显示创建状态（`Created xxx.md`）

> **注意：** 由于 Rust 版本目前不支持定时任务功能，因此不创建 `HEARTBEAT.md` 文件。

### 需求 4：Memory 子目录与文件创建

**用户故事：** 作为用户，我希望工作空间中有专门的记忆存储目录，以便 AI Agent 可以持久化存储重要信息。

#### 验收标准

1. WHEN 工作空间创建完成 THEN 系统 SHALL 创建 `memory` 子目录
2. IF `memory/MEMORY.md` 不存在 THEN 系统 SHALL 从模板创建该文件
3. IF `memory/HISTORY.md` 不存在 THEN 系统 SHALL 创建空的 `HISTORY.md` 文件
4. WHEN memory 目录创建成功 THEN 系统 SHALL 在控制台显示创建状态

### 需求 5：Skills 子目录创建

**用户故事：** 作为用户，我希望工作空间中有 skills 目录用于存放自定义技能，以便扩展 AI Agent 的能力。

#### 验收标准

1. WHEN 工作空间创建完成 THEN 系统 SHALL 创建 `skills` 子目录
2. IF skills 目录已存在 THEN 系统 SHALL 不执行任何操作（静默成功）

### 需求 6：模板内容一致性

**用户故事：** 作为用户，我希望 Rust 版创建的模板文件内容与 Python 版完全一致，以便迁移时不会丢失任何功能。

#### 验收标准

1. WHEN 创建模板文件 THEN 系统 SHALL 使用与 Python 版完全相同的文件内容
2. IF 模板文件包含占位符或说明文本 THEN 系统 SHALL 保持格式和内容不变

### 需求 7：错误处理与用户反馈

**用户故事：** 作为用户，我希望在模板创建过程中遇到错误时能收到清晰的反馈，以便了解问题并采取相应措施。

#### 验收标准

1. IF 文件创建失败 THEN 系统 SHALL 返回明确的错误信息，包含文件路径和错误原因
2. IF 目录创建失败 THEN 系统 SHALL 返回明确的错误信息，包含目录路径和错误原因
3. WHEN 发生任何错误 THEN 系统 SHALL 不中断整体流程，尽可能完成其他操作
