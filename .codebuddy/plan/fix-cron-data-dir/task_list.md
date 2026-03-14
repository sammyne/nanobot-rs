# 任务清单

## 概述

本任务清单用于实现需求文档中描述的 cron 数据目录路径修复工作。

## 任务列表

### 任务 1：修改 cron/mod.rs 中的 get_data_dir 函数

**状态：** 待执行

**优先级：** 高

**描述：** 将 `get_data_dir()` 函数从使用 `dirs::data_local_dir()` 改为使用 `nano_config::schema::HOME` 全局变量。

**涉及文件：** `crates/cli/src/commands/cron/mod.rs`

**具体修改：**

1. 添加导入语句：
   ```rust
   use nano_config::schema::HOME;
   ```

2. 修改 `get_data_dir()` 函数（第 52-54 行）：
   ```rust
   // 修改前：
   fn get_data_dir() -> std::path::PathBuf {
       dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("nanobot")
   }
   
   // 修改后：
   fn get_data_dir() -> std::path::PathBuf {
       HOME.join(".nanobot")
   }
   ```

**验收标准：**
- [ ] 函数返回 `$HOME/.nanobot` 路径
- [ ] 使用 `nano_config::schema::HOME` 全局变量
- [ ] 代码编译通过

---

### 任务 2：修改 cron/mod.rs 中的 init_cron_service 函数

**状态：** 待执行

**优先级：** 高

**描述：** 修改数据文件路径，将 cron 任务存储在 `$HOME/.nanobot/cron/jobs.json`。

**涉及文件：** `crates/cli/src/commands/cron/mod.rs`

**具体修改：**

修改 `init_cron_service()` 函数（第 57-65 行）：
```rust
// 修改前：
async fn init_cron_service() -> Result<Arc<CronService>> {
    let data_dir = get_data_dir();

    // 确保数据目录存在
    tokio::fs::create_dir_all(&data_dir).await.context("创建数据目录失败")?;

    let cron_file = data_dir.join("cron_jobs.json");

    CronService::new(cron_file).await.context("初始化 CronService 失败").map(Arc::new)
}

// 修改后：
async fn init_cron_service() -> Result<Arc<CronService>> {
    let data_dir = get_data_dir();
    let cron_dir = data_dir.join("cron");

    // 确保 cron 目录存在
    tokio::fs::create_dir_all(&cron_dir).await.context("创建 cron 目录失败")?;

    let cron_file = cron_dir.join("jobs.json");

    CronService::new(cron_file).await.context("初始化 CronService 失败").map(Arc::new)
}
```

**验收标准：**
- [ ] 数据文件路径为 `$HOME/.nanobot/cron/jobs.json`
- [ ] 自动创建 `cron` 子目录
- [ ] 错误信息包含上下文

---

### 任务 3：修改 gateway/mod.rs 中的 init_cron_service 函数

**状态：** 待执行

**优先级：** 高

**描述：** 将 gateway 模块中的 `init_cron_service()` 函数改为使用 `HOME` 全局变量，并保持与 cron 模块一致的路径结构。

**涉及文件：** `crates/cli/src/commands/gateway/mod.rs`

**具体修改：**

1. 添加导入语句（如果尚未存在）：
   ```rust
   use nano_config::schema::HOME;
   ```

2. 修改 `init_cron_service()` 函数（约第 195-211 行）：
   ```rust
   // 修改前：
   async fn init_cron_service(&self) -> Result<Arc<CronService>> {
       // 获取数据目录
       let data_dir = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join("nanobot");

       // 确保数据目录存在
       tokio::fs::create_dir_all(&data_dir).await.context("创建数据目录失败")?;

       let cron_file = data_dir.join("cron_jobs.json");
       info!("CronService 数据文件: {:?}", cron_file);

       // 创建 CronService
       let cron_service = Arc::new(CronService::new(cron_file).await.context("初始化 CronService 失败")?);

       Ok(cron_service)
   }

   // 修改后：
   async fn init_cron_service(&self) -> Result<Arc<CronService>> {
       // 获取数据目录
       let data_dir = HOME.join(".nanobot");
       let cron_dir = data_dir.join("cron");

       // 确保 cron 目录存在
       tokio::fs::create_dir_all(&cron_dir).await.context("创建 cron 目录失败")?;

       let cron_file = cron_dir.join("jobs.json");
       info!("CronService 数据文件: {:?}", cron_file);

       // 创建 CronService
       let cron_service = Arc::new(CronService::new(cron_file).await.context("初始化 CronService 失败")?);

       Ok(cron_service)
   }
   ```

**验收标准：**
- [ ] 使用 `HOME` 全局变量
- [ ] 数据文件路径为 `$HOME/.nanobot/cron/jobs.json`
- [ ] 代码编译通过

---

### 任务 4：移除 cli/Cargo.toml 中的 dirs 依赖

**状态：** 待执行

**优先级：** 中

**描述：** 从 `crates/cli/Cargo.toml` 中移除 `dirs` 依赖项。

**涉及文件：** `crates/cli/Cargo.toml`

**具体修改：**

```toml
# 修改前：
[dependencies]
nanobot-config.workspace = true
nanobot-provider.workspace = true
nanobot-agent.workspace = true
nanobot-channels.workspace = true
nanobot-templates.workspace = true
nanobot-cron.workspace = true
nanobot-heartbeat.workspace = true
nanobot-session.workspace = true
nanobot-subagent.workspace = true
clap.workspace = true
tokio = { workspace = true, features = ["signal"] }
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
dialoguer.workspace = true
dirs.workspace = true
chrono.workspace = true
async-trait = "0.1"

# 修改后：
[dependencies]
nanobot-config.workspace = true
nanobot-provider.workspace = true
nanobot-agent.workspace = true
nanobot-channels.workspace = true
nanobot-templates.workspace = true
nanobot-cron.workspace = true
nanobot-heartbeat.workspace = true
nanobot-session.workspace = true
nanobot-subagent.workspace = true
clap.workspace = true
tokio = { workspace = true, features = ["signal"] }
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
dialoguer.workspace = true
chrono.workspace = true
async-trait = "0.1"
```

**验收标准：**
- [ ] `dirs.workspace = true` 行已移除
- [ ] 项目编译通过

---

### 任务 5：验证编译和测试

**状态：** 待执行

**优先级：** 高

**描述：** 确保所有修改后代码能够正常编译，并运行相关测试。

**具体步骤：**

1. 运行 `cargo build` 确保编译通过
2. 运行 `cargo test -p nanobot-cli` 执行 CLI 模块测试
3. 手动测试 cron 命令：
   - `nanobot cron list`
   - `nanobot cron add --name "test" --message "test" --every 60`
   - 验证 `$HOME/.nanobot/cron/jobs.json` 文件已创建

**验收标准：**
- [ ] `cargo build` 编译成功
- [ ] 所有测试通过
- [ ] 手动测试验证数据文件路径正确

---

## 执行顺序

1. 任务 1 → 任务 2（cron/mod.rs 相关修改）
2. 任务 3（gateway/mod.rs 修改）
3. 任务 4（移除依赖）
4. 任务 5（验证）

## 风险与注意事项

1. **数据迁移：** 如果用户已有 `$HOME/.local/share/nanobot/cron_jobs.json` 文件，需要考虑数据迁移。本任务暂不处理数据迁移，用户需手动迁移。

2. **HOME 变量依赖：** `nano_config::schema::HOME` 在初始化时会 panic 如果无法获取用户主目录。这是预期行为，符合需求文档中的验收标准。

3. **路径一致性：** 确保所有模块使用相同的数据目录路径 `$HOME/.nanobot`。
