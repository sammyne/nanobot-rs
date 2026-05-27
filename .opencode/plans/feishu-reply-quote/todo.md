# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/config/src/schema/channel.rs` | 修改 | `FeishuConfig` 新增 `reply_to_message` 字段 |
| `crates/channels/src/feishu/mod.rs` | 修改 | `process_message()` 存 message_id 到 metadata；`send()` 条件分发 reply/create |

## 任务列表

### 1. FeishuConfig 新增 reply_to_message

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/channel.rs`
- 验收标准: `FeishuConfig` 有 `reply_to_message: bool` 字段，默认 `false`，JSON `"replyToMessage": true` 可正确反序列化
- 信心评估: 5
- 步骤:
  - [ ] 在 `FeishuConfig` 结构体中新增 `#[serde(default)] pub reply_to_message: bool`
  - [ ] `cargo check -p nanobot-config` 确认编译通过

### 2. process_message() 传递 message_id

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: `InboundMessage.metadata` 中包含 `"message_id"` 键，值为原始消息 ID
- 信心评估: 5（message_id 已在 process_message 中提取，只需 add_metadata 一行）
- 步骤:
  - [ ] 在构造 `InboundMessage` 处（约 line 366），链式调用 `.add_metadata("message_id", serde_json::Value::String(message_id.clone()))`（仅当 message_id 非空时）
  - [ ] `cargo check -p nanobot-channels` 确认编译通过

### 3. send() 条件分发 reply/create

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: `reply_to_message=true` 时首个内容块用 reply API（引用气泡），后续块用 create API；reply 失败回退 create；`reply_to_message=false` 时行为不变
- 风险/注意点: `reply_typed()` 的参数结构（`ReplyMessageQuery` + `ReplyMessageBody`）与 `send_typed()` 不同，需要查看 SDK 类型
- 信心评估: 4（SDK reply_typed 已有，但需确认参数构造方式）
- 步骤:
  - [ ] 在 `send()` 方法开头，从 `msg.metadata` 提取 `message_id`（`Option<String>`）
  - [ ] 判断是否使用 reply：`self.config.reply_to_message && message_id.is_some() && !msg.is_progress()`
  - [ ] 若使用 reply：构造 `ReplyMessageQuery { msg_type: Some("interactive"), .. }` 和 `ReplyMessageBody { content, uuid: None }`，调用 `self.client.im_v1_message().reply_typed(message_id, &query, &body, RequestOptions::default())`
  - [ ] 若 reply 成功，标记 `replied = true`，后续内容块（如有）走 create API
  - [ ] 若 reply 失败，`warn!` 记录并回退到 create API
  - [ ] `cargo check -p nanobot-channels` 确认编译通过
  - [ ] `cargo test -p nanobot-channels` 确认测试通过

## 实现建议

- `message_id` 从 `msg.metadata.get("message_id")` 提取，用 `and_then(|v| v.as_str())` 转为 `&str`
- reply API 的 `msg_type` 通过 `ReplyMessageQuery` 传递（不在 body 中），参考 SDK 的 `ReplyMessageQuery` 结构
- 现有 `send()` 中构造 `content_json` 的逻辑可复用，reply 和 create 共享同一个 content 构造
