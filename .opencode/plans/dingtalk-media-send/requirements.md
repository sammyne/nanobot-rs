# 需求

## 目标与背景

钉钉 channel 的 `send()` 当前只能发送 markdown 文本（通过 session webhook 的 `reply_markdown`），完全忽略 `OutboundMessage.media` 字段。MessageTool 已实现（PR #103），LLM 可以填充 `media` 字段，但钉钉端无法投递媒体内容。

对齐 Python 版 PR #1337，为钉钉 channel 增加图片、音频、视频、文件的发送能力。

## 方案比较（强制）

### 方案 1: Markdown 内嵌图片 URL（最小可行版）

- 思路: 仅处理 HTTP URL 图片，在 markdown 文本中拼接 `![image](url)`，通过现有 `reply_markdown` 发送
- 优点: 零新 API 调用，改动极小（约 10 行）
- 缺点: 不支持本地文件、不支持非图片附件（音频/视频/文件）、不支持 media_id
- 工作量估算: S

### 方案 2: Upload + Batch Send（理想架构）

- 思路: 文本保持 `reply_markdown`（session webhook）；媒体文件通过 `upload_to_dingtalk` 上传获取 media_id，再通过 `oToMessages/batchSend` API 发送。HTTP URL 图片优先直接发（`sampleImageMsg` + `photoURL`），失败再 upload fallback
- 优点: 支持所有媒体类型（图片/音频/视频/文件）、支持 HTTP URL 和本地文件、与 Python 版行为对齐
- 缺点: 需要实现 batch send API 调用（SDK 未提供）、需要 `sender_staff_id`（企业 bot 场景下始终存在）
- 工作量估算: M

### 推荐

方案 2。MessageTool 的 `media` 字段可以是本地文件路径或 URL，只支持 URL 图片覆盖面太窄。batch send API 调用本身很简单（一个 `post_json` 调用），SDK 已提供 upload 和 token 管理。

## 功能需求列表

### 核心功能

1. **媒体类型识别**：根据文件扩展名判断上传类型（image/voice/video/file）
2. **媒体字节读取**：支持从 HTTP(S) URL 下载和本地文件读取，返回字节、文件名、MIME 类型
3. **媒体上传**：调用 SDK 的 `ChatbotReplier::upload_to_dingtalk()` 上传媒体文件，获取 media_id
4. **Batch Send 消息发送**：调用 `oToMessages/batchSend` API，支持 `sampleMarkdown`、`sampleImageMsg`、`sampleFile` 消息类型
5. **媒体发送 fallback 链**：
   - HTTP URL 图片 → `sampleImageMsg` + `photoURL`
   - 失败 → 读取字节 → upload → `sampleImageMsg` + `mediaId`（图片）
   - 非图片 → 读取字节 → upload → `sampleFile` + `mediaId`
6. **send() 重构**：先发文本（保持 `reply_markdown`），再逐个发媒体；媒体发送失败时发送可见 fallback 文本

### 扩展功能

- 无

## 非功能需求

- **性能**：媒体逐个串行发送（钉钉 API 限流），不阻塞文本发送
- **安全**：本地文件路径不做额外限制（workspace 限制已在 MessageTool 层处理）
- **兼容性**：无媒体时行为与当前完全一致（只发 markdown）
- **可维护性**：媒体相关逻辑提取为独立方法，不膨胀 `send()`
- **测试要求**：单元测试覆盖类型识别、文件名猜测、fallback 逻辑；不测试实际 API 调用（需要钉钉凭证）

## 边界与不做事项

- 不修改 `OutboundMessage` 结构
- 不修改飞书 channel 的 `send()`（飞书媒体发送是独立需求）
- 不实现 progress 消息的媒体发送（progress 消息只有文本）
- 不实现群聊 batch send（当前 `sender_staff_id` 仅适用于单聊 oToMessages）

## 假设与约束

- **技术假设**：企业 bot 场景下 `ChatbotMessage.sender_staff_id` 始终存在；`sender_staff_id` 缺失时 media 发送静默跳过并 warn 日志
- **资源约束**：SDK 的 `upload_to_dingtalk` 和 `HttpClient::post_json` 已可用，无需引入新依赖
- **环境约束**：batch send API 需要企业内部应用权限（`chatbot.send.oToMessage`），README 需补充权限说明

## 待确认事项

- 无
