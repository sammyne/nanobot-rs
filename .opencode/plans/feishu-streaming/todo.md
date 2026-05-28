# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/config/src/schema/channel.rs` | 修改 | `FeishuConfig` 新增 `streaming: bool` 字段 |
| `crates/channels/src/feishu/mod.rs` | 修改 | 新增 `StreamBuf`、CardKit API 方法、`send()` 流式路由逻辑 |
| `crates/channels/src/feishu/tests.rs` | 修改 | 新增流式相关测试 |

## 任务列表

### 1. ✅ FeishuConfig 新增 streaming 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/channel.rs`
- 验收标准: `cargo check -p nanobot-config` 通过；`streaming` 默认 true
- 风险/注意点: `FeishuConfig` 当前使用 `#[derive(Default)]`，`bool` 默认为 false，需要手写 `Default` 或用 `#[serde(default)]` 配合默认函数
- 信心评估: 5
- 步骤:
  - [ ] `FeishuConfig` 新增 `#[serde(default = "default_true")] pub streaming: bool` 字段（复用已有的 `default_true` 函数，如果没有则新增）
  - [ ] 由于 `FeishuConfig` derive `Default`（bool 默认 false），需改为手写 `Default` impl 设置 `streaming: true`，或改用 `#[serde(default)]` 在 struct 级别 + 手写 Default
  - [ ] 运行 `cargo check -p nanobot-config` 验证通过

### 2. ✅ Feishu 通道实现 CardKit 流式输出

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/mod.rs`、`crates/channels/src/feishu/tests.rs`
- 验收标准: `cargo test -p nanobot-channels` 通过；进度消息走流式卡片路径；最终响应关闭流式模式
- 风险/注意点: CardKit API 需要飞书应用有 `cardkit:card:write` 权限；API 调用失败时需回退为普通卡片；更新节流 ~0.5s 避免限流
- 信心评估: 3（CardKit API 的请求/响应格式需要在实现时确认）
- 步骤:
  - [ ] 定义 `StreamBuf` 结构体：`text: String`、`card_id: Option<String>`、`sequence: u32`、`last_edit: Instant`
  - [ ] `Feishu` 结构体新增 `stream_bufs: Arc<RwLock<HashMap<String, StreamBuf>>>` 字段，在 `new()` 和 `Clone` 中初始化
  - [ ] 实现 `create_streaming_card(&self, chat_id: &str) -> Option<String>`：调用 `POST /open-apis/cardkit/v1/cards` 创建卡片（`schema: "2.0"`, `streaming_mode: true`, `update_multi: true`），包含一个空 markdown 元素（`element_id: "streaming_md"`）；然后调用 `im.v1.message.create` 发送卡片到聊天；返回 `card_id`
  - [ ] 实现 `stream_update_text(&self, card_id: &str, text: &str, sequence: u32)`：调用 `PUT /open-apis/cardkit/v1/cards/{card_id}/elements/streaming_md/content` 更新 markdown 内容
  - [ ] 实现 `close_streaming_mode(&self, card_id: &str, sequence: u32)`：调用 `PATCH /open-apis/cardkit/v1/cards/{card_id}/settings` 设置 `streaming_mode: false`
  - [ ] 修改 `send()` 方法：
    - 若 `streaming` 配置为 false 或消息不是进度消息：走原有逻辑
    - 若消息是进度消息（`is_progress()`）：获取或创建 `StreamBuf`；首次创建流式卡片；累积文本；节流后更新卡片内容
    - 若消息不是进度消息且存在活跃的 `StreamBuf`：发送最终内容更新、关闭流式模式、移除 `StreamBuf`；然后走原有发送逻辑发送完整响应
  - [ ] 在 `tests.rs` 中新增测试：`streaming=false` 时不创建 StreamBuf；StreamBuf 的 sequence 递增逻辑
  - [ ] 运行 `cargo test -p nanobot-channels` 验证通过

### 3. ✅ 全量验证

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: 无
- 验收标准: `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 通过；`cargo test` 全工作空间通过
- 信心评估: 5
- 步骤:
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`
  - [ ] 修复发现的问题

## 实现建议

- CardKit API 通过 `self.client.request()` 发送原生 HTTP 请求（feishu-sdk 的 `Client` 支持通用 HTTP 请求）
- 流式卡片的 markdown 元素 `element_id` 固定为 `"streaming_md"`，与上游一致
- 更新节流：检查 `last_edit.elapsed() >= Duration::from_millis(500)`，不满足则跳过本次更新（文本已累积，下次更新时会包含）
- CardKit 创建失败时 warn 日志并回退为普通卡片（设置 `stream_buf.card_id = None` 作为标记）
- `send()` 中检测到非进度消息时，先关闭流式卡片再发送最终响应，确保用户看到完整内容
