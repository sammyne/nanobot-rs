# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/config/src/schema/channel.rs` | 修改 | 在 `FeishuConfig` 中新增 `react_emoji` 字段 |
| `crates/channels/src/feishu/mod.rs` | 修改 | 在 `process_message()` 中提取 `message_id` 并调用 reaction API |
| `crates/channels/src/feishu/tests.rs` | 修改 | 新增 `react_emoji` 配置和反序列化测试 |

## 任务列表

### 1. ✅ FeishuConfig 新增 react_emoji 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/channel.rs`
- 验收标准: `FeishuConfig` 反序列化时，缺省 `reactEmoji` 字段得到默认值 `"THUMBSUP"`；显式设置为空字符串时得到空字符串
- 风险/注意点: 无。新增可选字段，`#[serde(default)]` 保证向后兼容
- 信心评估: 5
- 步骤:
  - [ ] 在 `channel.rs` 末尾（`default_send_progress` 函数附近）添加 `default_react_emoji` 函数，返回 `"THUMBSUP".to_string()`
  - [ ] 在 `FeishuConfig` 结构体的 `allow_from` 字段之后，新增 `react_emoji: String` 字段，标注 `#[serde(default = "default_react_emoji")]`，文档注释为 `/// 收到消息时添加的表情回应类型（为空则禁用）`

### 2. ✅ process_message 中添加表情回应

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: 飞书 channel 收到授权用户的消息后，立即异步调用 reaction API 添加配置的表情；`react_emoji` 为空时跳过；API 调用失败仅 warn 日志不阻塞消息处理
- 风险/注意点: `tokio::spawn` 的 task 需要 `'static` 生命周期，`Client` 和 `String` 都实现了 `Send + 'static`，可以 move 进闭包
- 信心评估: 4（feishu-sdk 的 `im_v1_reaction().create()` API 未在本项目中使用过，需确认 `OperationBuilder::send()` 的错误类型）
- 步骤:
  - [ ] 在 `process_message()` 中解析 `msg_event` 之后（约第 106 行后），提取 `message_id`：`let message_id = msg_event.message.message_id.clone();`
  - [ ] 在权限检查通过之后、日志行之前（约第 186 行和第 188 行之间），插入 reaction 调用逻辑：检查 `self.config.react_emoji` 非空且 `message_id` 为 `Some`，则 clone `self.client`、`react_emoji`、`message_id`，`tokio::spawn` 异步调用 reaction API，失败时 `warn!("添加表情回应失败: {e}")`
  - [ ] reaction API 调用方式：`client.im_v1_reaction().create().path_param("message_id", &mid).body_value(serde_json::json!({"reaction_type": {"emoji_type": emoji}})).send().await`
  - [ ] 运行 `cargo clippy -p nanobot-channels -- -D warnings -D clippy::uninlined_format_args` 确认无警告

### 3. ✅ 添加测试

- 优先级: P1
- 依赖项: 1, 2
- 涉及文件: `crates/channels/src/feishu/tests.rs`
- 验收标准: 新增测试覆盖 `react_emoji` 的默认值、显式配置、空值禁用三种场景；所有现有测试继续通过
- 风险/注意点: 现有测试中构造 `FeishuConfig` 时未包含 `react_emoji` 字段，由于使用 `#[serde(default)]`，结构体字面量构造需要补上该字段或改用 `..Default::default()`。但 `FeishuConfig` derive 了 `Default`，`Default::default()` 会给 `react_emoji` 空字符串而非 `"THUMBSUP"`（`Default` 不走 serde default）。因此现有测试需要显式补上 `react_emoji: "THUMBSUP".to_string()` 字段
- 信心评估: 5
- 步骤:
  - [ ] 更新所有现有测试中的 `FeishuConfig` 结构体字面量，补上 `react_emoji: "THUMBSUP".to_string()` 字段（共 8 处：`feishu_channel_creation`、`feishu_channel_validation_empty_app_id`、`feishu_channel_validation_empty_app_secret`、`permission_check_with_whitelist`、`permission_check_empty_whitelist`、`channel_name`、`channel_running_state`、`channel_clone`）
  - [ ] 新增测试 `react_emoji_default_value`：用 JSON `{"appId":"x","appSecret":"y"}` 反序列化 `FeishuConfig`，断言 `react_emoji == "THUMBSUP"`
  - [ ] 新增测试 `react_emoji_custom_value`：用 JSON `{"appId":"x","appSecret":"y","reactEmoji":"SMILE"}` 反序列化，断言 `react_emoji == "SMILE"`
  - [ ] 新增测试 `react_emoji_empty_disables`：用 JSON `{"appId":"x","appSecret":"y","reactEmoji":""}` 反序列化，断言 `react_emoji.is_empty()`
  - [ ] 更新现有 `feishu_config_serialization` 测试的 `FeishuConfig` 构造，补上 `react_emoji` 字段，并在断言中增加 `assert_eq!(config.react_emoji, deserialized.react_emoji)`
  - [ ] 运行 `cargo test -p nanobot-channels` 确认全部通过

## 实现建议

- reaction API 调用通过 `feishu-sdk` 已有的 `client.im_v1_reaction().create()` 完成，无需引入新依赖
- fire-and-forget 模式参考 `process_message()` 中图片下载失败的处理方式（`error!` 日志 + 继续执行）
- `tokio::spawn` 内的闭包需要 move 所有权，clone `Client`（已实现 `Clone`）、`String` 即可
