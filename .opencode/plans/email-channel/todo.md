# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `Cargo.toml` | 修改 | 添加 `imap`、`lettre`、`mail-parser`、`native-tls` 工作空间依赖 |
| `crates/channels/Cargo.toml` | 修改 | 引用新增的工作空间依赖 |
| `crates/config/src/schema/channel.rs` | 修改 | 添加 `EmailConfig` 结构体 |
| `crates/config/src/schema/mod.rs` | 修改 | 导出 `EmailConfig` |
| `crates/config/src/lib.rs` | 修改 | 重新导出 `EmailConfig` |
| `crates/config/src/schema/tests.rs` | 修改 | 添加 `EmailConfig` 测试 |
| `crates/channels/src/email/mod.rs` | 新增 | Email 通道实现（Channel trait、IMAP 轮询、SMTP 发送） |
| `crates/channels/src/email/tests.rs` | 新增 | 单元测试 |
| `crates/channels/src/lib.rs` | 修改 | 添加 `email` 模块声明和重新导出 |
| `crates/channels/src/manager/mod.rs` | 修改 | 注册 email 通道 |
| `crates/channels/AGENTS.md` | 修改 | 更新文档 |

## 任务列表

### 1. ✅ 添加外部依赖

- 优先级: P0
- 依赖项: 无
- 涉及文件: `Cargo.toml`、`crates/channels/Cargo.toml`
- 验收标准: `cargo check -p nanobot-channels` 通过
- 风险/注意点: `imap` crate 需要 `native-tls` feature 支持 SSL 连接；`lettre` 需要 `tokio1` + `native-tls` features 支持异步发送
- 信心评估: 5
- 步骤:
  - [ ] 在 `Cargo.toml` 的 `[workspace.dependencies]` 中添加 `native-tls = "0.2"`
  - [ ] 在 `Cargo.toml` 中添加 `[workspace.dependencies.imap]`，version `"3"`，features `["native-tls"]`
  - [ ] 在 `Cargo.toml` 中添加 `[workspace.dependencies.lettre]`，version `"0.11"`，features `["tokio1", "tokio1-native-tls", "smtp-transport", "builder"]`
  - [ ] 在 `Cargo.toml` 中添加 `mail-parser = "0.9"`
  - [ ] 在 `crates/channels/Cargo.toml` 的 `[dependencies]` 中添加 `imap.workspace = true`、`lettre.workspace = true`、`mail-parser.workspace = true`、`native-tls.workspace = true`、`regex.workspace = true`
  - [ ] 运行 `cargo check -p nanobot-channels` 验证编译通过

### 2. ✅ 添加 EmailConfig 到 config crate

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/channel.rs`、`crates/config/src/schema/mod.rs`、`crates/config/src/lib.rs`、`crates/config/src/schema/tests.rs`
- 验收标准: `cargo test -p nanobot-config` 通过；`EmailConfig` 可从 `nanobot_config` 导入
- 风险/注意点: 字段命名使用 camelCase（`#[serde(rename_all = "camelCase")]`）与现有通道配置一致；默认值需与上游 Python 对齐
- 信心评估: 5
- 步骤:
  - [ ] 在 `channel.rs` 中添加 `EmailConfig` 结构体，字段包括：`enabled: bool`（默认 false）、`consent_granted: bool`（默认 false）、`imap_host: String`、`imap_port: u16`（默认 993）、`imap_username: String`、`imap_password: String`、`imap_mailbox: String`（默认 "INBOX"）、`imap_use_ssl: bool`（默认 true）、`smtp_host: String`、`smtp_port: u16`（默认 587）、`smtp_username: String`、`smtp_password: String`、`smtp_use_tls: bool`（默认 true）、`smtp_use_ssl: bool`（默认 false）、`from_address: String`、`auto_reply_enabled: bool`（默认 true）、`poll_interval_seconds: u64`（默认 30）、`mark_seen: bool`（默认 true）、`max_body_chars: usize`（默认 12000）、`subject_prefix: String`（默认 "Re: "）、`allow_from: Vec<String>`、`verify_dkim: bool`（默认 true）、`verify_spf: bool`（默认 true）
  - [ ] 为 `EmailConfig` 实现 `validate()` 方法：启用时检查 `imap_host`、`imap_username`、`imap_password`、`smtp_host`、`smtp_username`、`smtp_password` 非空
  - [ ] 在 `ChannelsConfig` 中添加 `email: EmailConfig` 字段（`#[serde(default)]`）
  - [ ] 在 `schema/mod.rs` 的 `pub use channel::` 中添加 `EmailConfig`
  - [ ] 在 `lib.rs` 的重新导出列表中添加 `EmailConfig`
  - [ ] 在 `tests.rs` 中添加测试：默认值正确、camelCase 序列化/反序列化、validate 检查必填字段
  - [ ] 运行 `cargo test -p nanobot-config` 验证通过

### 3. ✅ 创建 email 模块骨架并集成到 ChannelManager

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/channels/src/email/mod.rs`、`crates/channels/src/email/tests.rs`、`crates/channels/src/lib.rs`、`crates/channels/src/manager/mod.rs`
- 验收标准: `cargo check -p nanobot-channels` 通过；ChannelManager 在 `config.email.enabled` 为 true 时创建 Email 通道实例
- 风险/注意点: Email struct 需要持有 IMAP/SMTP 配置、inbound_tx、运行状态等；start() 中启动轮询循环需要 spawn 后台任务
- 信心评估: 5
- 步骤:
  - [ ] 创建 `crates/channels/src/email/mod.rs`，定义 `Email` 结构体，字段：`config: EmailConfig`、`running: Arc<RwLock<bool>>`、`task_handle: Arc<RwLock<Option<JoinHandle<()>>>>`、`name: String`（固定 "email"）、`inbound_tx: mpsc::Sender<InboundMessage>`、`self_addresses: HashSet<String>`、`last_subject_by_chat: Arc<RwLock<HashMap<String, String>>>`、`last_message_id_by_chat: Arc<RwLock<HashMap<String, String>>>`、`processed_uids: Arc<RwLock<HashSet<String>>>`
  - [ ] 实现 `Email::new(config: EmailConfig, inbound_tx: mpsc::Sender<InboundMessage>) -> ChannelResult<Self>`，初始化 self_addresses 集合（从 from_address、smtp_username、imap_username 收集并归一化）
  - [ ] 实现 Channel trait 的 5 个方法（start/stop/send 先留空实现，is_running 和 name 完整实现）
  - [ ] 创建 `crates/channels/src/email/tests.rs`，添加 `use super::*;`
  - [ ] 在 `lib.rs` 中添加 `pub mod email;` 和 `pub use email::Email;`
  - [ ] 在 `manager/mod.rs` 中：import `EmailConfig` 和 `Email`；在 `ChannelManager::new()` 中添加 `if manager.config.email.enabled { manager.add_email_channel(...) }`；实现 `add_email_channel` 方法（参照 `add_feishu_channel`）
  - [ ] 运行 `cargo check -p nanobot-channels` 验证通过

### 4. ✅ 实现邮件解析工具函数

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/email/mod.rs`、`crates/channels/src/email/tests.rs`
- 验收标准: 所有解析函数的单元测试通过
- 风险/注意点: `mail-parser` 的 `Message` 类型解析 raw bytes；HTML-to-text 只需简单的标签剥离（与上游一致）；SPF/DKIM 用正则匹配 Authentication-Results 头
- 信心评估: 4
- 步骤:
  - [ ] 实现 `html_to_text(raw_html: &str) -> String`：将 `<br>` 替换为 `\n`，`</p>` 替换为 `\n`，剥离所有 HTML 标签，HTML 实体解码（使用 `html_escape::decode_html_entities` 或手动处理 `&amp;`/`&lt;`/`&gt;`/`&quot;`/`&#...;`）
  - [ ] 实现 `extract_text_body(message: &mail_parser::Message) -> String`：遍历 MIME 部分，优先取 text/plain，其次取 text/html 并调用 `html_to_text`，跳过 attachment disposition 的部分
  - [ ] 实现 `check_authentication_results(message: &mail_parser::Message) -> (bool, bool)`：获取所有 `Authentication-Results` 头，用正则 `\bspf\s*=\s*pass\b` 和 `\bdkim\s*=\s*pass\b` 匹配，返回 `(spf_pass, dkim_pass)`
  - [ ] 实现 `normalize_address(value: &str) -> String`：提取邮件地址部分（处理 `"Name <addr>"` 格式），转小写，去空白
  - [ ] 实现 `is_self_address(&self, sender: &str) -> bool`：归一化后检查是否在 `self_addresses` 中
  - [ ] 实现 `is_allowed(&self, sender: &str) -> bool`：`allow_from` 为空返回 true（允许所有）；否则检查发件人是否在白名单中（大小写不敏感）
  - [ ] 在 `tests.rs` 中添加测试：`html_to_text` 处理 br/p 标签和实体；`extract_text_body` 处理纯文本、HTML、multipart 邮件；`check_authentication_results` 处理 pass/fail/缺失场景；`normalize_address` 处理各种格式；`is_self_address` 和 `is_allowed` 的边界情况
  - [ ] 运行 `cargo test -p nanobot-channels` 验证通过

### 5. ✅ 实现 IMAP 轮询收件

- 优先级: P0
- 依赖项: 3, 4
- 涉及文件: `crates/channels/src/email/mod.rs`、`crates/channels/src/email/tests.rs`
- 验收标准: `Channel::start()` 启动后台轮询任务；轮询逻辑能连接 IMAP、搜索未读、解析邮件、发送 InboundMessage；UID 去重正常工作
- 风险/注意点: IMAP 操作在 `spawn_blocking` 中执行；UID 集合需要上限防止无限增长（上限 100000，超出时淘汰前半部分）；`consent_granted` 为 false 时 start() 直接返回并 warn
- 信心评估: 3（`imap` crate API 细节需要在实现时确认，特别是 UID 提取和 SSL 连接方式）
- 步骤:
  - [ ] 实现 `fetch_messages(&self, search_criteria: &[&str], mark_seen: bool, dedupe: bool, limit: usize) -> Vec<ParsedEmail>` 私有方法：包含重试逻辑（调用 `fetch_messages_once`，捕获 stale 错误后重试一次）
  - [ ] 实现 `fetch_messages_once(...)` 私有方法：创建 IMAP 连接（SSL 或明文）→ login → select mailbox → search → 遍历结果 fetch `(BODY.PEEK[] UID)` → 用 `mail-parser` 解析 → 提取 sender/subject/date/message_id/body → 检查自身地址 → 检查 SPF/DKIM → 检查 allow_from → UID 去重 → 构建 `ParsedEmail` → mark_seen → logout
  - [ ] 定义 `ParsedEmail` 辅助结构体：`sender: String`、`subject: String`、`message_id: String`、`content: String`（含 `[EMAIL-CONTEXT]` 前缀）、`metadata: HashMap<String, Value>`
  - [ ] 实现 `is_stale_imap_error(err: &imap::Error) -> bool`：检查错误消息是否包含 stale 连接标记（"disconnected for inactivity"、"eof occurred"、"socket error"、"connection reset"、"broken pipe"、"bye"）
  - [ ] 实现 `is_missing_mailbox_error(err: &imap::Error) -> bool`：检查错误消息是否包含邮箱不存在标记
  - [ ] 实现 `remember_processed_uid(&self, uid: &str, dedupe: bool, cycle_uids: &mut HashSet<String>)`：添加到 cycle_uids；若 dedupe 则添加到 `self.processed_uids`，超过 100000 时淘汰前半部分
  - [ ] 实现 `Channel::start()`：检查 `consent_granted`；若 `verify_dkim` 和 `verify_spf` 均为 false 则 warn；spawn 后台任务循环调用 `fetch_new_messages` + `sleep(poll_interval_seconds)`；每封邮件构建 `InboundMessage` 并通过 `inbound_tx` 发送，同时更新 `last_subject_by_chat` 和 `last_message_id_by_chat`
  - [ ] 实现 `Channel::stop()`：设置 `running = false`，abort 后台任务
  - [ ] 在 `tests.rs` 中添加测试：`is_stale_imap_error` 和 `is_missing_mailbox_error` 的匹配逻辑；`remember_processed_uid` 的淘汰逻辑；`ParsedEmail` 的 content 格式（`[EMAIL-CONTEXT]` 前缀）
  - [ ] 运行 `cargo test -p nanobot-channels` 验证通过

### 6. ✅ 实现 SMTP 发送回复

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/channels/src/email/mod.rs`、`crates/channels/src/email/tests.rs`
- 验收标准: `Channel::send()` 能构建邮件并通过 SMTP 发送；邮件线程头（In-Reply-To、References）正确设置
- 风险/注意点: `lettre` 的 `AsyncSmtpTransport` 需要 tokio runtime；需区分 SSL（端口 465 直连 TLS）和 STARTTLS（端口 587 先明文再升级）两种模式
- 信心评估: 4
- 步骤:
  - [ ] 实现 `reply_subject(&self, base_subject: &str) -> String`：如果 subject 已以 "re:" 开头（大小写不敏感）则原样返回，否则加上 `subject_prefix`
  - [ ] 实现 `Channel::send()`：检查 `consent_granted`；检查 `smtp_host` 非空；提取 `to_addr = msg.chat_id`；判断是否为回复（`to_addr` 在 `last_subject_by_chat` 中）；若是回复且 `auto_reply_enabled` 为 false 且无 `force_send` metadata 则跳过；构建 subject（从 `last_subject_by_chat` 获取或默认 "nanobot reply"，支持 metadata 中的 subject 覆盖）；用 `lettre::Message::builder()` 构建邮件（From、To、Subject、In-Reply-To、References、纯文本 body）；根据 `smtp_use_ssl`/`smtp_use_tls` 选择 transport 模式发送
  - [ ] 实现 `build_smtp_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>>`：根据配置创建 SMTP transport（SSL 直连 / STARTTLS / 明文），设置认证凭据
  - [ ] 在 `tests.rs` 中添加测试：`reply_subject` 处理已有 "Re:" 前缀和无前缀的情况；send 方法在 `consent_granted=false` 时跳过
  - [ ] 运行 `cargo test -p nanobot-channels` 验证通过

### 7. ✅ 实现历史邮件查询

- 优先级: P1
- 依赖项: 5
- 涉及文件: `crates/channels/src/email/mod.rs`、`crates/channels/src/email/tests.rs`
- 验收标准: `fetch_messages_between_dates` 方法能按日期范围查询邮件
- 风险/注意点: IMAP 日期格式为 `DD-Mon-YYYY`（英文月份缩写）；`end_date <= start_date` 时返回空
- 信心评估: 5
- 步骤:
  - [ ] 实现 `format_imap_date(date: &chrono::NaiveDate) -> String`：格式化为 `DD-Mon-YYYY`（月份使用英文缩写数组 `["Jan", "Feb", ..., "Dec"]`）
  - [ ] 实现 `pub fn fetch_messages_between_dates(&self, start_date: NaiveDate, end_date: NaiveDate, limit: usize) -> Vec<ParsedEmail>`：检查 `end_date > start_date`；构建 IMAP 搜索条件 `SINCE <start> BEFORE <end>`；调用 `fetch_messages` 并传入 `mark_seen=false, dedupe=false`
  - [ ] 在 `tests.rs` 中添加测试：`format_imap_date` 格式正确；`end_date <= start_date` 返回空
  - [ ] 运行 `cargo test -p nanobot-channels` 验证通过

### 8. ✅ 更新 AGENTS.md

- 优先级: P2
- 依赖项: 3, 4, 5, 6, 7
- 涉及文件: `crates/channels/AGENTS.md`
- 验收标准: 文档准确反映新增的 Email 通道
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在架构图中添加 `Email` 节点（`│ Email  │` / `│IMAP+SMTP│`）
  - [ ] 在关键类型列表中添加 `**Email**` -- IMAP 轮询 + SMTP 回复通道实现
  - [ ] 更新 crate 顶部描述，将"钉钉、飞书"改为"钉钉、飞书、Email"

## 实现建议

- 参照 `crates/channels/src/feishu/mod.rs` 的结构模式：`new()` 初始化 → `Channel::start()` spawn 后台任务 → `Channel::stop()` abort 任务 → `Channel::send()` 构建并发送消息
- IMAP 操作全部在 `tokio::task::spawn_blocking` 中执行，与上游 Python 的 `asyncio.to_thread` 模式一致
- `ChannelError` 枚举已有足够的变体（`StartFailed`、`SendFailed`、`Config`、`Network` 等），无需新增
- `InboundMessage` 和 `OutboundMessage` 的 `metadata` 字段用于传递 `message_id`、`subject`、`force_send` 等邮件特有信息
- `regex` crate 已在工作空间依赖中，可直接用于 SPF/DKIM 验证的正则匹配
- 邮件内容格式化为 `[EMAIL-CONTEXT] Email received.\nFrom: ...\nSubject: ...\nDate: ...\n\n<body>`，与上游一致
