# 实施计划

- [ ] 1. 创建 cmd 模块基础结构
  - 在 crates/agent/src 目录下创建 cmd 子目录
  - 创建 cmd/mod.rs 文件作为模块入口
  - 在 mod.rs 中添加 `#[cfg(test)] mod tests;` 引入测试模块
  - _需求：1.1、1.2、1.3、1.4_

- [ ] 2. 实现 Command 基 trait
  - 在 cmd/mod.rs 中定义 Command trait
  - 定义 async run 方法签名，接收 AgentLoop、InboundMessage、session_key 等参数
  - 指定 run 方法返回类型为 Result<String, String>
  - _需求：2.1、2.2、2.3、2.4_

- [ ] 3. 实现 HelpCmd 结构体
  - 在 cmd/help.rs 中定义 HelpCmd 结构体
  - 为 HelpCmd 实现 Command trait
  - 实现 HelpCmd::run 方法，返回帮助信息字符串
  - 确保返回的帮助信息与原有实现一致
  - _需求：3.1、3.2、3.3、3.4_

- [ ] 4. 实现 NewCmd 结构体
  - 在 cmd/new.rs 中定义 NewCmd 结构体
  - 为 NewCmd 实现 Command trait
  - 实现 NewCmd::run 方法，包含记忆整合逻辑（archive_all=true）
  - 在 NewCmd::run 中实现会话清除逻辑
  - 添加并发控制，避免重复整合
  - 返回成功或错误消息
  - _需求：4.1、4.2、4.3、4.4、4.5、4.6_

- [ ] 5. 在 cmd 模块中声明子模块
  - 在 cmd/mod.rs 中添加 pub mod help;
  - 在 cmd/mod.rs 中添加 pub mod new;
  - 重新导出 Command trait 和命令结构体（如需要）
  - _需求：3.5、4.7_

- [ ] 6. 重构 loop 模块的命令分发逻辑
  - 读取 crates/agent/src/loop/mod.rs 中的 try_handle_cmd 方法
  - 移除 try_handle_cmd 中的直接命令处理逻辑
  - 根据命令名称从相应子模块构建对应的 Cmd 结构体实例
  - 调用 Cmd 结构体的 run 方法
  - 将 run 方法的返回值（Result<String, String>）转换为 OutboundMessage
  - 保持对不支持命令的错误处理逻辑
  - _需求：5.1、5.2、5.3、5.4、5.5_

- [ ] 7. 编写 HelpCmd 单元测试
  - 在 cmd/help.rs 中添加单元测试
  - 验证返回的帮助信息内容
  - 遵循项目测试命名规范（不使用 test_ 前缀）
  - _需求：6.2、6.5_

- [ ] 8. 编写 NewCmd 单元测试
  - 在 cmd/new.rs 中添加单元测试
  - 验证会话清除逻辑
  - 验证记忆整合逻辑
  - 遵循项目测试命名规范（不使用 test_ 前缀）
  - _需求：6.3、6.5_

- [ ] 9. 编写命令模块集成测试
  - 创建 cmd/tests.rs 文件
  - 验证命令分发到正确命令结构体
  - 验证命令执行结果正确返回
  - 验证不支持命令的错误处理
  - _需求：6.1、7.1、7.2、7.3_
