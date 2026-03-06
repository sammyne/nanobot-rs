# 需求文档

## 引言

本需求文档描述了为 `nanobot-skills` crate 实现对 `builtin` 文件夹内置 skills 的支持功能。当前 `SkillsLoader` 的 builtin 目录路径是基于用户 workspace 计算的（`workspace/builtin-skills`），而非使用 crate 自身携带的 `crates/skills/builtin/` 目录中的内置 skills。本功能采用**方案B**实现：首次运行时将 crate 的内置 skills 自动复制到 `workspace/builtin-skills` 目录，使用户可以查看、理解和自定义这些内置 skills，同时保持与 workspace skills 的优先级机制。

为确保内置 skills 与 crate 版本的一致性，本功能引入**版本管理机制**：在 `workspace/builtin-skills` 目录中维护 `VERSION` 文件，记录创建该目录的 skills crate 版本。当检测到版本不匹配时，系统将自动更新内置 skills 以确保与当前 crate 版本一致。

**技术实现关键点：** 由于 `CARGO_MANIFEST_DIR` 环境变量仅在编译时设置，运行时无法定位 crate 源码目录，因此采用 `include_dir` crate 在**编译时将 `builtin/` 目录内容嵌入到二进制文件中**。运行时从嵌入资源提取文件到 `workspace/builtin-skills`，解决了部署后无法找到 crate 内置资源的问题。

## 需求

### 需求 1：版本管理与目录初始化

**用户故事：** 作为 nanobot 应用开发者，我希望内置 skills 能够自动与 crate 版本保持同步，以便确保功能的正确性和一致性。

#### 验收标准

1. WHEN `SkillsLoader` 首次初始化 AND `workspace/builtin-skills` 目录不存在 THEN 系统 SHALL 将 crate 的 `builtin/` 目录内容复制到 `workspace/builtin-skills`，并创建 `VERSION` 文件记录当前 crate 版本
2. WHEN 创建 `VERSION` 文件 THEN 系统 SHALL 将 skills crate 的版本号（从 `Cargo.toml` 获取）写入该文件
3. WHEN `SkillsLoader` 初始化 AND `workspace/builtin-skills` 目录已存在 THEN 系统 SHALL 读取 `VERSION` 文件并与当前 crate 版本比较
4. IF `workspace/builtin-skills/VERSION` 文件不存在或版本不匹配 THEN 系统 SHALL 删除整个 `workspace/builtin-skills` 目录，并重新复制 crate 的 `builtin/` 目录内容，同时更新 `VERSION` 文件
5. WHEN 内置 skills 复制完成 THEN 系统 SHALL 保持原有的目录结构（包括子目录如 `scripts/`）
6. WHEN 复制或删除过程中发生错误 THEN 系统 SHALL 记录错误日志并继续运行（优雅降级）

### 需求 2：内置 Skills 运行时加载

**用户故事：** 作为 SkillsLoader 的使用者，我希望能够从 workspace/builtin-skills 目录加载内置 skills，以便应用程序能够提供开箱即用的 skills 功能。

#### 验收标准

1. WHEN `SkillsLoader` 初始化 THEN 系统 SHALL 能够识别并注册 `workspace/builtin-skills` 目录下的所有 skills
2. WHEN 调用 `list_skills` 方法 THEN 系统 SHALL 返回包含内置 skills 的完整列表（从 `workspace/builtin-skills` 加载）
3. WHEN 调用 `load_skill` 加载一个内置 skill THEN 系统 SHALL 从 `workspace/builtin-skills` 目录读取并返回正确的 skill 内容
4. WHEN 调用 `load_skills_for_context` 包含内置 skill 名称 THEN 系统 SHALL 正确加载并格式化该 skill 的内容

### 需求 3：Skills 优先级管理

**用户故事：** 作为 nanobot 用户，我希望 workspace 中的自定义 skills 能够覆盖内置 skills，以便我可以自定义或扩展内置功能。

#### 验收标准

1. WHEN `workspace/skills` 目录中的 skill 与 `workspace/builtin-skills` 中的 skill 同名 THEN 系统 SHALL 优先返回 `workspace/skills` 中的 skill
2. WHEN 用户修改了 `workspace/builtin-skills` 中的 skill AND 版本未发生变化 THEN 系统 SHALL 使用修改后的版本
3. WHEN crate 版本升级导致内置 skills 更新 THEN 系统 SHALL 删除整个 `workspace/builtin-skills` 目录（包括用户的修改），并重新复制新版本的内置 skills
4. WHEN 调用相关方法查询 skill 信息 THEN 系统 SHALL 正确报告 skill 的来源路径

### 需求 4：API 兼容性与扩展

**用户故事：** 作为现有代码的使用者，我希望新的内置 skills 功能不破坏现有的 API 接口，以便现有代码能够平滑升级。

#### 验收标准

1. WHEN 使用现有的 `SkillsLoader::new(workspace)` 构造函数 THEN 系统 SHALL 继续正常工作并自动初始化内置 skills（包括版本检查）
2. IF 内置 skills 初始化失败（如权限问题）THEN 系统 SHALL 优雅降级并记录警告日志，不影响其他 skills 的加载
3. WHEN 内置 skills 初始化或版本更新 THEN 系统 SHALL 支持可选的日志输出以帮助调试

### 需求 5：测试覆盖

**用户故事：** 作为项目维护者，我希望内置 skills 功能有完整的测试覆盖，以便确保功能的稳定性和正确性。

#### 验收标准

1. WHEN 运行测试 THEN 系统 SHALL 包含验证内置 skills 初始化（首次复制）的单元测试
2. WHEN 运行测试 THEN 系统 SHALL 包含验证版本匹配检查的单元测试
3. WHEN 运行测试 THEN 系统 SHALL 包含验证版本不匹配时自动更新的集成测试
4. WHEN 运行测试 THEN 系统 SHALL 包含验证 workspace skill 与 builtin skill 优先级的集成测试
