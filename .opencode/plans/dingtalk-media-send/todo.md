# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/channels/src/dingtalk/media.rs` | 新增 | 媒体辅助函数：类型识别、字节读取、batch send、fallback 发送 |
| `crates/channels/src/dingtalk/mod.rs` | 修改 | 声明 `mod media;`，重构 `send()` 处理媒体 |
| `crates/channels/src/dingtalk/tests.rs` | 修改 | 新增媒体辅助函数的单元测试 |

## 任务列表

### ✅ 1. 创建 media.rs：辅助函数和常量

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/dingtalk/media.rs`
- 验收标准: 编译通过，所有辅助函数可从 `mod.rs` 调用
- 风险/注意点: 扩展名集合需与 Python 版对齐
- 信心评估: 5
- 步骤:
  - [ ] 创建 `crates/channels/src/dingtalk/media.rs`
  - [ ] 定义常量 `IMAGE_EXTS`、`AUDIO_EXTS`、`VIDEO_EXTS`（使用 `&[&str]` 切片）
  - [ ] 实现 `pub(super) fn is_http_url(path: &str) -> bool`：检查 `http://` 或 `https://` 前缀（忽略大小写）
  - [ ] 实现 `pub(super) fn guess_upload_type(media_ref: &str) -> &'static str`：从 URL path 或本地路径提取扩展名，匹配常量返回 `"image"` / `"voice"` / `"video"` / `"file"`
  - [ ] 实现 `pub(super) fn guess_filename(media_ref: &str, upload_type: &str) -> String`：从路径/URL 提取文件名，无法提取时按 upload_type 返回 fallback（`image.jpg` / `audio.amr` / `video.mp4` / `file.bin`）
  - [ ] 实现 `pub(super) fn guess_mime_type(filename: &str) -> &'static str`：根据扩展名返回 MIME 类型，未知返回 `"application/octet-stream"`

### ✅ 2. 添加媒体字节读取函数

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/dingtalk/media.rs`
- 验收标准: HTTP URL 和本地文件路径均能正确读取字节、文件名、MIME 类型
- 风险/注意点: HTTP 下载失败应返回 Err 而非 panic；本地文件使用 `tokio::fs::read` 避免阻塞
- 信心评估: 5
- 步骤:
  - [ ] 实现 `pub(super) async fn read_media_bytes(media_ref: &str) -> ChannelResult<(Vec<u8>, String, String)>`，返回 (字节, 文件名, MIME 类型)
  - [ ] HTTP URL 分支：`reqwest::get(url)` 下载，从 `Content-Type` header 提取 MIME，用 `guess_filename` 获取文件名
  - [ ] 本地文件分支：支持 `file://` 协议（URL decode path），普通路径直接读取；用 `guess_mime_type` 获取 MIME
  - [ ] 错误映射为 `ChannelError::SendFailed`

### ✅ 3. 添加 batch send API 调用

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/dingtalk/media.rs`
- 验收标准: 能通过 `HttpClient::post_json` 调用 `oToMessages/batchSend` API
- 风险/注意点: API URL 是 `https://api.dingtalk.com/v1.0/robot/oToMessages/batchSend`；`HttpClient::post_json` 的第三个参数 `access_token: Option<&str>` 会自动设置 `x-acs-dingtalk-access-token` header
- 信心评估: 4（API 格式来自 Python 版，未直接验证 Rust SDK 的 post_json header 行为）
- 步骤:
  - [ ] 实现 `pub(super) async fn send_batch_message(http_client: &HttpClient, token: &str, robot_code: &str, staff_id: &str, msg_key: &str, msg_param: &serde_json::Value) -> ChannelResult<()>`
  - [ ] 构造请求体：`{ "robotCode": robot_code, "userIds": [staff_id], "msgKey": msg_key, "msgParam": json_string(msg_param) }`
  - [ ] 调用 `http_client.post_json(BATCH_SEND_URL, &body, Some(token))`
  - [ ] 检查返回值中的 `processQueryKey`（成功标志），失败时 warn 日志并返回 `ChannelError::SendFailed`

### ✅ 4. 添加媒体发送函数（含 fallback 链）

- 优先级: P0
- 依赖项: 1, 2, 3
- 涉及文件: `crates/channels/src/dingtalk/media.rs`
- 验收标准: HTTP URL 图片优先直接发，失败 fallback 到 upload；非图片走 upload + sampleFile
- 风险/注意点: `ChatbotReplier::upload_to_dingtalk` 返回 `dingtalk_stream::Result<String>`，需映射错误
- 信心评估: 4
- 步骤:
  - [ ] 实现 `pub(super) async fn send_media_ref(replier: &ChatbotReplier, http_client: &HttpClient, token: &str, robot_code: &str, staff_id: &str, media_ref: &str) -> ChannelResult<()>`
  - [ ] 空 media_ref 直接返回 Ok
  - [ ] 判断 upload_type，若为 `"image"` 且 `is_http_url`：尝试 `send_batch_message` with `sampleImageMsg` + `{"photoURL": media_ref}`，成功则返回
  - [ ] 调用 `read_media_bytes` 获取字节
  - [ ] 调用 `replier.upload_to_dingtalk(&bytes, upload_type, &filename, &mime)` 获取 media_id
  - [ ] 若 upload_type 为 `"image"`：尝试 `send_batch_message` with `sampleImageMsg` + `{"photoURL": media_id}`（钉钉 sampleImageMsg 的 photoURL 字段也接受 media_id）
  - [ ] 若图片 media_id 发送失败或非图片：`send_batch_message` with `sampleFile` + `{"mediaId": media_id, "fileName": filename, "fileType": ext}`

### ✅ 5. 重构 send() 处理媒体

- 优先级: P0
- 依赖项: 4
- 涉及文件: `crates/channels/src/dingtalk/mod.rs`
- 验收标准: 有媒体时逐个发送，失败时发 fallback 文本；无媒体时行为不变
- 风险/注意点: `sender_staff_id` 为 None 时跳过媒体发送并 warn；文本为空时跳过 reply_markdown
- 信心评估: 5
- 步骤:
  - [ ] 在 `mod.rs` 顶部添加 `mod media;`
  - [ ] 重构 `send()` 方法：
    - 获取 `incoming_msg`（现有逻辑不变）
    - 创建 `replier`（现有逻辑不变）
    - 文本非空时调用 `replier.reply_markdown`（现有逻辑不变）
    - 若 `msg.media` 非空：
      - 从 `incoming_msg.sender_staff_id` 提取 staff_id，为 None 则 warn 并跳过
      - 调用 `self.token_manager.get_access_token().await` 获取 token
      - 遍历 `msg.media`，对每个 media_ref 调用 `media::send_media_ref`
      - 发送失败时通过 `replier.reply_markdown` 发送 `[Attachment send failed: {filename}]` fallback 文本

### ✅ 6. 单元测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/channels/src/dingtalk/tests.rs`
- 验收标准: 覆盖所有辅助函数的边界情况
- 风险/注意点: 只测试纯函数，不测试需要网络的 async 函数
- 信心评估: 5
- 步骤:
  - [ ] 测试 `is_http_url`：http/https/HTTP/大小写混合 → true；本地路径/file:///空字符串 → false
  - [ ] 测试 `guess_upload_type`：.jpg → image，.mp3 → voice，.mp4 → video，.pdf → file，无扩展名 → file，HTTP URL 带查询参数
  - [ ] 测试 `guess_filename`：本地路径提取文件名，HTTP URL 提取文件名（含 URL decode），无文件名时 fallback
  - [ ] 测试 `guess_mime_type`：.png → image/png，.mp3 → audio/mpeg，未知扩展名 → application/octet-stream

### ✅ 7. 全量验证

- 优先级: P0
- 依赖项: 1-6
- 涉及文件: 全部
- 验收标准: `cargo +nightly fmt`、`cargo clippy -- -D warnings -D clippy::uninlined_format_args`、`cargo test` 全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 运行 `cargo +nightly fmt`
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`
  - [ ] 确认无媒体时 send() 行为不变（回归检查）

## 实现建议

- `HttpClient::post_json(url, &body, Some(token))` 自动设置 `x-acs-dingtalk-access-token` header，无需手动构造 header
- `ChatbotReplier::upload_to_dingtalk(&bytes, filetype, filename, mimetype)` 已封装 `oapi.dingtalk.com/media/upload`，直接复用
- `TokenManager::get_access_token()` 有内置缓存（提前 5 分钟过期），可在 send() 中安全调用
- MIME 类型映射只需覆盖 IMAGE_EXTS + AUDIO_EXTS + VIDEO_EXTS 中的扩展名，其余返回 `application/octet-stream`
- `reqwest` 已是 channels crate 的依赖，无需新增
