# 需求

## 目标与背景

当 gateway 运行时，CLI 命令（`nanobot cron add/remove/enable`）创建独立的 `CronService` 实例修改同一个 `jobs.json` 文件。Gateway 的内存缓存（`RwLock<CronStore>`）不知道文件被外部修改，下次 `save()` 时用旧数据覆盖 CLI 的修改，导致数据丢失。

合并上游两个配套修复：
- HKUDS/nanobot PR #1375：添加 mtime 追踪，文件变化时重载
- HKUDS/nanobot PR #1399：在每次 timer tick 开头触发重载

## 方案比较（强制）

### 方案 1: 在 CronStorage 中添加 mtime 追踪（最小可行版）

- 思路: `CronStorage` 新增 `last_mtime` 字段，`save()` 后更新 mtime，新增 `reload_if_changed()` 方法比较文件 mtime 并按需重载。timer tick 循环中在检查 due jobs 前调用 `reload_if_changed()`
- 优点: 改动集中在 cron crate 内部，不影响公共 API 签名；mtime 比较开销极低（一次 `fs::metadata()` 系统调用）
- 缺点: mtime 精度依赖文件系统（通常 1 秒），极端情况下同一秒内的修改可能被遗漏
- 工作量估算: S

### 方案 2: 文件监听（inotify/kqueue）（理想架构）

- 思路: 使用 `notify` crate 监听 `jobs.json` 的文件变更事件，实时触发重载
- 优点: 零延迟感知文件变更，无轮询开销
- 缺点: 新增 `notify` 依赖，跨平台行为差异（Linux inotify vs macOS kqueue vs Windows ReadDirectoryChanges），对于 cron 场景（秒级精度）过度设计
- 工作量估算: M

### 推荐

方案 1。cron 任务本身是秒级精度，mtime 的 1 秒精度完全足够。与上游 Python 版实现方式一致。

## 功能需求列表

### 核心功能

1. `CronStorage` 新增 `last_mtime: RwLock<SystemTime>` 字段（初始值 `SystemTime::UNIX_EPOCH`），追踪 `jobs.json` 的最后修改时间
2. `CronStorage::load()` 中加载文件后记录 mtime
3. `CronStorage::save()` 中写入文件后更新 `last_mtime`
4. 新增 `CronStorage::reload_if_changed()` 方法：
   - 获取文件当前 mtime
   - 与 `last_mtime` 比较，不一致时重新读取文件内容并替换内存 store
   - 更新 `last_mtime`
   - 文件不存在或读取失败时保持现有内存数据不变
5. `CronService` 的 timer tick 循环中，在 `storage.list_jobs()` 之前调用 `storage.reload_if_changed().await`

### 扩展功能

- 无

## 非功能需求

- **性能**: `reload_if_changed()` 仅调用 `tokio::fs::metadata()` 获取 mtime，不读取文件内容（除非 mtime 变化）。每次 timer tick 一次，开销可忽略
- **安全**: 无影响
- **兼容性**: 向后兼容，现有 API 签名不变
- **测试要求**: 新增测试验证外部修改 jobs.json 后 `reload_if_changed()` 能正确重载

## 边界与不做事项

- 不使用文件监听（inotify/kqueue）
- 不修改 `CronService` 或 `CronStorage` 的公共 API 签名
- 不处理 mtime 相同但内容不同的极端情况（同一秒内多次修改）

## 假设与约束

- **技术假设**: 文件系统 mtime 精度至少为 1 秒（所有主流 OS 均满足）
- **资源约束**: 无
- **环境约束**: 无

## 待确认事项

- 无
