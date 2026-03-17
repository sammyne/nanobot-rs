# 实施计划

- [ ] 1. 在 AgentLoop 中实现 `try_handle_cmd` 方法框架
   - 在 `crates/agent/src/loop/mod.rs` 中为 `AgentLoop` 添加 `try_handle_cmd` 方法
   - 方法签名：`async fn try_handle_cmd(&self, msg: InboundMessage) -> Result<OutboundMessage, InboundMessage>`
   - 返回值含义：
     - `Ok(_)`: 是命令，并已正确处理
     - `Err(_)`: 不是命令，返回入参的 InboundMessage
   - 使用 `match` 结构处理不同命令，预留扩展空间
   - _需求：1.2、1.4、5.2_

- [ ] 2. 实现命令识别和解析逻辑
   - 在 `try_handle_cmd` 方法开始处检查消息内容是否以 `/` 开头
   - 如果不是命令，返回 `Err(msg)` 将消息返回给调用者
   - 如果是命令，提取命令名称（去除前导 `/`），使用 `to_lowercase()` 和 `trim()` 处理
   - _需求：2.1、2.2、2.3_

- [ ] 3. 实现 `/help` 命令处理逻辑
   - 在 `try_handle_cmd` 的 `match` 分支中添加 `/help` 命令处理
   - 返回 `Ok(OutboundMessage)`，内容为：`🐈 nanobot commands:\n/new — Start a new conversation\n/help — Show available commands`
   - 正确设置 `channel` 和 `chat_id` 字段
   - 对于未知命令，返回 `Ok(OutboundMessage)` 包含提示信息
   - _需求：3.1、3.2、3.3_

- [ ] 4. 修改 `process_message` 方法集成命令处理
   - 在 `process_message` 方法开始处调用 `try_handle_cmd` 方法
   - 使用 `match` 处理返回值：
     - `Ok(outbound)`: 直接返回命令处理结果
     - `Err(msg)`: 继续执行现有的消息处理流程（LLM 调用等）
   - _需求：1.1、1.3、4.1、4.2_

- [ ] 5. 添加单元测试
   - 在 `crates/agent/src/loop/tests.rs` 中添加测试用例
   - 测试 `/help` 命令识别和响应
   - 测试大小写不敏感（`/HELP`、`/Help`）
   - 测试空格处理（`/help `）
   - 测试非命令消息正常处理
   - _需求：2.1、2.2、2.3、4.1_
