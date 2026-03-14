# 需求文档

## 引言

本需求文档描述了对 `crates/cli` 模块中 cron 相关数据目录路径的修复工作。当前实现使用了 `dirs::data_local_dir()` 返回系统级数据目录（如 `$HOME/.local/share/nanobot`），但用户期望使用用户主目录下的 `.nanobot` 隐藏目录作为统一的数据存储位置。

## 需求

### 需求 1：修复 get_data_dir 函数返回路径

**用户故事：** 作为 CLI 用户，我希望所有 nanobot 数据统一存储在 `$HOME/.nanobot` 目录下，以便于管理和备份。

#### 验收标准

1. WHEN 调用 `get_data_dir()` 函数 THEN 系统 SHALL 返回 `$HOME/.nanobot` 路径
2. IF 用户主目录无法获取 THEN 系统 SHALL 直接 panic，提示用户主目录不存在
3. WHEN 路径返回后 THEN 系统 SHALL 确保路径格式为 `~/.nanobot`（或 `/home/用户名/.nanobot`）

### 需求 2：修复 cron_service 数据文件路径

**用户故事：** 作为 CLI 用户，我希望 cron 任务数据存储在 `$HOME/.nanobot/cron/jobs.json`，以便与项目其他模块保持一致的目录结构。

#### 验收标准

1. WHEN 初始化 CronService THEN 系统 SHALL 将数据文件存储在 `$HOME/.nanobot/cron/jobs.json`
2. WHEN 创建 cron 目录 THEN 系统 SHALL 在数据目录下创建 `cron` 子目录
3. WHEN 数据文件路径确定后 THEN 系统 SHALL 使用 `jobs.json` 作为文件名而非 `cron_jobs.json`

### 需求 3：确保目录自动创建

**用户故事：** 作为 CLI 用户，我希望在首次使用 cron 功能时系统能自动创建所需的目录结构，以便无需手动配置。

#### 验收标准

1. WHEN 初始化 CronService 且 `$HOME/.nanobot` 目录不存在 THEN 系统 SHALL 自动创建该目录
2. WHEN 初始化 CronService 且 `$HOME/.nanobot/cron` 目录不存在 THEN 系统 SHALL 自动创建该目录
3. IF 目录创建失败 THEN 系统 SHALL 返回包含上下文信息的错误

### 需求 4：移除 dirs 库依赖

**用户故事：** 作为项目维护者，我希望移除对 dirs 库的依赖，以便使用项目内部已定义的 HOME 全局变量，保持依赖最小化。

#### 验收标准

1. WHEN 完成 get_data_dir 函数重构 THEN 系统 SHALL 不再依赖 dirs 库的任何功能
2. WHEN 重构完成后 THEN 系统 SHALL 从 `crates/cli/Cargo.toml` 中移除 dirs 依赖项
3. WHEN 发现其他模块使用 dirs 库 THEN 系统 SHALL 重构这些模块使用 HOME 全局变量
4. WHEN 所有模块重构完成 THEN 系统 SHALL 从整个项目的 Cargo.toml 中完全移除 dirs 依赖

## 技术说明

### 涉及文件

1. `crates/cli/src/commands/cron/mod.rs`
   - `get_data_dir()` 函数需要修改返回 `$HOME/.nanobot`
   - `init_cron_service()` 函数需要修改数据文件路径为 `$HOME/.nanobot/cron/jobs.json`

### 实现建议

使用 `nano_config::schema::HOME` 全局变量（已在 nano-config 中定义）获取用户主目录，然后拼接 `.nanobot` 子目录。

```rust
use nano_config::schema::HOME;

fn get_data_dir() -> std::path::PathBuf {
    HOME.join(".nanobot")
}
```

对于 cron 数据文件：

```rust
async fn init_cron_service() -> Result<Arc<CronService>> {
    let data_dir = get_data_dir();
    let cron_dir = data_dir.join("cron");

    // 确保目录存在
    tokio::fs::create_dir_all(&cron_dir).await.context("创建 cron 目录失败")?;

    let cron_file = cron_dir.join("jobs.json");
    
    CronService::new(cron_file).await.context("初始化 CronService 失败").map(Arc::new)
}
```
