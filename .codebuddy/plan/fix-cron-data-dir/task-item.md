# 实施计划

- [ ] 1. 修改 get_data_dir 函数实现
   - 将 `get_data_dir()` 函数修改为使用 `nano_config::schema::HOME` 全局变量
   - 返回 `$HOME/.nanobot` 路径，移除对 `dirs::data_local_dir()` 的调用
   - 添加错误处理：当 HOME 无法获取时 panic
   - _需求：1.1、1.2、1.3_

- [ ] 2. 修改 init_cron_service 函数的数据文件路径
   - 修改数据文件路径为 `$HOME/.nanobot/cron/jobs.json`
   - 在 `cron_dir` 变量中拼接 `cron` 子目录
   - 将文件名从 `cron_jobs.json` 改为 `jobs.json`
   - _需求：2.1、2.2、2.3_

- [ ] 3. 添加目录自动创建逻辑
   - 在 `init_cron_service()` 中使用 `tokio::fs::create_dir_all()` 创建 cron 目录
   - 添加错误上下文信息，使用 `.context()` 提供友好的错误提示
   - 确保在 CronService 初始化前目录已存在
   - _需求：3.1、3.2、3.3_

- [ ] 4. 移除 dirs 库依赖
   - 从 `crates/cli/Cargo.toml` 中移除 dirs 依赖项
   - 检查并移除代码中所有 `use dirs::*` 相关的导入语句
   - 运行 `cargo build` 确认编译通过
   - _需求：4.1、4.2_

- [ ] 5. 检查并清理项目中其他模块的 dirs 依赖
   - 使用 `grep` 搜索项目中所有使用 dirs 库的地方
   - 如有其他模块使用，重构为使用 HOME 全局变量
   - 从项目根 `Cargo.toml` 中移除 dirs 依赖（如果存在）
   - _需求：4.3、4.4_

- [ ] 6. 编译测试与验证
   - 运行 `cargo build` 确保项目编译通过
   - 运行 `cargo test` 确保现有测试通过
   - 手动测试 cron 功能，验证数据文件存储在正确路径
   - _需求：全部_
