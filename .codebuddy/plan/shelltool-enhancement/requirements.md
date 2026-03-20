# 需求文档

## 引言
本需求文档描述了 Rust 版 ShellTool 功能补全的需求。参照 Python 版 ExecTool 的实现，为 Rust 版 ShellTool 添加更完善的安全机制和配置选项，使其具备与 Python 版同等的功能水平。

## 需求

### 需求 1：正则表达式拒绝模式列表

**用户故事：** 作为一名系统管理员，我希望能够使用正则表达式配置危险命令的拒绝模式，以便更灵活地拦截潜在危险的 Shell 命令。

#### 验收标准
1. WHEN 初始化 ShellTool 时 THEN 系统 SHALL 支持自定义 `deny_patterns` 参数（正则表达式列表）
2. WHEN 未提供 `deny_patterns` 时 THEN 系统 SHALL 使用默认的拒绝模式列表（包含 rm -rf、dd if=、mkfs、fork bomb 等危险命令）
3. WHEN 执行命令时 THEN 系统 SHALL 使用正则匹配检查命令是否匹配任一拒绝模式
4. IF 命令匹配任一拒绝模式 THEN 系统 SHALL 拒绝执行并返回错误信息

### 需求 2：允许模式白名单

**用户故事：** 作为一名系统管理员，我希望能够配置命令白名单，以便在严格模式下只允许执行特定的命令。

#### 验收标准
1. WHEN 初始化 ShellTool 时 THEN 系统 SHALL 支持配置 `allow_patterns` 参数（正则表达式列表）
2. IF 配置了 `allow_patterns` 且列表非空 THEN 系统 SHALL 只允许匹配白名单模式的命令执行
3. WHEN 命令不匹配任何允许模式 THEN 系统 SHALL 拒绝执行并返回错误信息
4. WHEN 未配置 `allow_patterns` 或列表为空 THEN 系统 SHALL 不进行白名单检查

### 需求 3：工作空间路径限制

**用户故事：** 作为一名安全审计员，我希望能够限制命令只能在工作空间目录内执行，以便防止恶意命令访问或修改工作空间外的文件。

#### 验收标准
1. WHEN 初始化 ShellTool 时 THEN 系统 SHALL 支持配置 `restrict_to_workspace` 布尔参数
2. IF `restrict_to_workspace` 为 true THEN 系统 SHALL 检测命令中的路径遍历尝试（如 `../`、`..\\`）
3. IF 检测到路径遍历尝试 THEN 系统 SHALL 拒绝执行并返回错误信息
4. IF `restrict_to_workspace` 为 true THEN 系统 SHALL 提取命令中的绝对路径并验证其是否在工作空间内
5. IF 命令中的绝对路径位于工作空间外 THEN 系统 SHALL 拒绝执行并返回错误信息

### 需求 4：PATH 环境变量扩展

**用户故事：** 作为一名开发者，我希望能够在执行命令时追加自定义的 PATH 路径，以便使用特定目录下的工具或程序。

#### 验收标准
1. WHEN 初始化 ShellTool 时 THEN 系统 SHALL 支持配置 `path_append` 字符串参数
2. IF 配置了 `path_append` THEN 系统 SHALL 在执行命令时将其追加到 PATH 环境变量
3. WHEN 追加 PATH 时 THEN 系统 SHALL 使用系统正确的路径分隔符（Unix 为 `:`，Windows 为 `;`）

### 需求 5：绝对路径提取辅助方法

**用户故事：** 作为一名开发者，我希望系统能够从命令字符串中提取绝对路径，以便进行路径安全检查。

#### 验收标准
1. WHEN 系统需要检查路径安全时 THEN 系统 SHALL 提供方法从命令中提取 Windows 风格绝对路径（如 `C:\...`）
2. WHEN 系统需要检查路径安全时 THEN 系统 SHALL 提供方法从命令中提取 POSIX 风格绝对路径（如 `/...`）
3. WHEN 提取路径时 THEN 系统 SHALL 正确处理带引号和空格的路径

### 需求 6：统一的安全守卫方法

**用户故事：** 作为一名开发者，我希望系统有一个统一的安全守卫方法来整合所有安全检查，以便代码结构清晰且易于维护。

#### 验收标准
1. WHEN 执行命令前 THEN 系统 SHALL 调用统一的安全守卫方法
2. WHEN 安全守卫方法执行时 THEN 系统 SHALL 按顺序执行：拒绝模式检查、允许模式检查、工作空间限制检查
3. IF 任一安全检查失败 THEN 系统 SHALL 返回相应的错误信息并终止命令执行
4. WHEN 所有安全检查通过 THEN 系统 SHALL 允许命令继续执行
