# 实施计划：模块 1 - 基础对话能力

## 任务清单

- [ ] 1. 项目结构和依赖配置
   - 创建 Cargo 工作空间和 crate 结构（cli、config、provider、agent）
   - 添加依赖：tokio、clap、serde、serde_json、async-openai、tracing、anyhow、thiserror
   - 配置 Cargo.toml 的 features 和优化选项
   - _需求：技术选型_

- [ ] 2. 实现配置管理模块
   - 定义配置数据结构（ProviderConfig、Config）并实现 serde 序列化
   - 实现配置加载函数，支持从 `~/.nanobot/config.json` 读取
   - 实现配置验证逻辑（检查 base_url、api_key、model 字段）
   - 实现配置保存函数，设置文件权限为 600
   - 编写配置模块的单元测试
   - _需求：3.1、3.2、3.3、3.4_

- [ ] 3. 实现日志记录模块
   - 配置 tracing 订阅器，输出结构化日志到 stderr
   - 实现日志级别控制（支持 RUST_LOG 环境变量）
   - 实现敏感信息脱敏处理器（API Key 等）
   - _需求：6.1、6.2、6.3、6.4_

- [ ] 4. 实现 CLI 框架
   - 使用 clap 定义命令结构（main、onboard、agent 子命令）
   - 实现 `--help` 帮助信息显示
   - 实现命令路由和错误处理
   - 设置正确的退出码（成功返回 0，失败返回非 0）
   - _需求：5.1、5.2、5.3、5.4、5.5_

- [ ] 5. 实现 LLM 提供者抽象层
   - 定义 Provider trait（chat 方法）
   - 实现统一的 OpenAI Provider（使用 async-openai 库，支持自定义 base_url）
   - 设置默认 base_url 为 OpenAI 官方地址
   - 配置请求超时为 120 秒
   - 编写 Provider 的单元测试（使用 mock）
   - _需求：4.1、4.2、4.3、4.4_

- [ ] 6. 实现 onboard 命令
   - 实现交互式配置向导（提示用户输入 base_url、API Key、模型名称）
   - 设置 base_url 默认值为 OpenAI 官方地址
   - 调用配置管理模块保存配置到 `~/.nanobot/config.json`
   - 处理配置文件已存在的情况（提示是否覆盖）
   - _需求：1.1、1.2、1.3、1.4、1.5_

- [ ] 7. 实现 agent 命令
   - 加载配置并初始化 LLM Provider
   - 实现交互式 REPL 循环（读取用户输入）
   - 维护对话历史上下文（消息列表）
   - 调用 LLM Provider 获取回复并显示
   - 处理退出命令（exit、quit）
   - 处理 LLM 调用失败情况（显示错误并允许继续）
   - _需求：2.1、2.2、2.3、2.4、2.5、2.6_

- [ ] 8. 集成测试和文档完善
   - 编写 onboard 命令的集成测试
   - 编写 agent 命令的集成测试
   - 为所有公开 API 添加文档注释
   - 更新 README.md 说明使用方法
   - _需求：可维护性要求_
