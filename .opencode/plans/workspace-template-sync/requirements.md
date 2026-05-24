# 需求

## 目标与背景

用户升级 nanobot 后，新版本可能新增了模板文件（如 `TOOLS.md`）。当前只有 `nanobot onboard` 会创建模板文件，而 `agent`/`gateway` 命令启动时不做任何检查。虽然运行时缺失模板文件不会崩溃（`ContextBuilder::load_bootstrap_files` 静默跳过），但用户无法发现新模板的存在，也就无法自定义。

本需求在 `agent` 和 `gateway` 命令启动时自动补全缺失的模板文件，仅创建不存在的文件，不覆盖用户已有的自定义内容。对齐上游 Python 版 PR #1253。

## 方案比较（强制）

### 方案 1: 硬编码模板列表（最小可行版）

- 思路: 在 `crates/nanobot/src/utils/mod.rs` 新增 `sync_workspace_templates` 函数，内部硬编码模板列表（与 `WorkspaceInitializer::create_root_templates` 相同），逐个调用 `nanobot_templates::xxx_template()` 获取内容并写入缺失文件。`onboard` 改用此函数。
- 优点: 改动最小，不涉及 templates crate 的 API 变更
- 缺点: 新增模板时需要同时更新三处（模板文件 + templates crate accessor + sync 函数列表），容易遗漏
- 工作量估算: S

### 方案 2: 模板枚举 API（理想架构）

- 思路: 在 `nanobot-templates` crate 新增 `all_templates() -> impl Iterator<Item = (&str, &str)>` 函数，利用 `include_dir::Dir::files()` 自动枚举所有嵌入的模板文件，返回 `(相对路径, 内容)` 对。sync 函数遍历此迭代器创建缺失文件。新增模板只需在 `templates/` 目录放文件，无需修改任何 Rust 代码。
- 优点: 新增模板零代码改动，单一事实来源
- 缺点: 需要修改 templates crate 的公共 API（但改动很小，约 5 行）
- 工作量估算: S

### 推荐

方案 2。额外改动量极小（templates crate 加一个函数），但消除了"三处同步"的维护负担。这正是 `include_dir` 的设计意图。

## 功能需求列表

### 核心功能

- `nanobot-templates` crate 新增 `all_templates()` 函数，返回所有嵌入模板的 `(相对路径, 内容)` 迭代器
- `crates/nanobot/src/utils/mod.rs` 新增 `sync_workspace_templates(workspace: &Path) -> Result<Vec<String>>` 函数：
  - 确保 workspace 目录及必要子目录（`memory/`）存在
  - 遍历 `all_templates()`，对每个模板文件检查是否已存在，不存在则创建
  - 返回新创建的文件名列表
- `AgentCmd::run()` 在加载配置后、初始化 provider 前调用 `sync_workspace_templates`
- `GatewayCmd::run()` 在加载配置后、初始化 provider 前调用 `sync_workspace_templates`
- `OnboardCmd::run()` 中 `WorkspaceInitializer` 改用 `sync_workspace_templates` 处理模板文件部分
- 有新文件被创建时，打印日志通知用户（如 `[info] Synced 2 new workspace templates: TOOLS.md, HEARTBEAT.md`）

### 扩展功能

- 无

## 非功能需求

- **性能**：启动时同步为同步 I/O（模板文件数量固定且少，无需异步），不影响启动速度
- **安全**：仅写入 workspace 目录内的文件，不覆盖已有文件
- **兼容性**：完全向后兼容，已有工作空间不受影响
- **可维护性**：新增模板只需在 `crates/templates/templates/` 放文件，无需修改 Rust 代码
- **测试要求**：为 `sync_workspace_templates` 和 `all_templates` 编写单元测试

## 边界与不做事项

- 不处理模板内容更新（已有文件不覆盖，即使内容过时）
- 不处理模板删除（旧版本创建的文件不会被新版本删除）
- 不在 `cron` 命令中调用（cron 是短期任务管理命令，不涉及 agent 运行）
- `skills/` 目录和 `memory/HISTORY.md`（空文件）的创建仍由 `WorkspaceInitializer` 负责，不纳入模板同步

## 假设与约束

- **技术假设**：`include_dir::Dir::files()` 仅返回当前层级文件，不递归子目录。枚举 `memory/MEMORY.md` 需通过 `dirs()` 递归遍历子目录
- **资源约束**：无
- **环境约束**：无

## 待确认事项

- 无
