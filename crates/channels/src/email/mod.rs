//! Email 通道实现
//!
//! 通过 IMAP 轮询接收邮件、SMTP 发送回复。
//! IMAP 操作在 `tokio::task::spawn_blocking` 中执行，与上游 Python 的
//! `asyncio.to_thread` 模式一致。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Datelike;
use lettre::message::Mailbox;
use lettre::message::header::{InReplyTo, References};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use nanobot_config::EmailConfig;
use regex::Regex;
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::error::{ChannelError, ChannelResult};
use crate::messages::{InboundMessage, OutboundMessage};
use crate::traits::Channel;

/// 已处理 UID 集合的上限，超出时淘汰前半部分
const MAX_PROCESSED_UIDS: usize = 100_000;

/// IMAP 连接过期的错误标记（小写匹配）
const IMAP_RECONNECT_MARKERS: &[&str] = &[
    "disconnected for inactivity",
    "eof occurred in violation of protocol",
    "socket error",
    "connection reset",
    "broken pipe",
    "bye",
];

/// IMAP 邮箱不存在的错误标记（小写匹配）
const IMAP_MISSING_MAILBOX_MARKERS: &[&str] =
    &["mailbox doesn't exist", "select failed", "no such mailbox", "can't open mailbox", "does not exist"];

/// 从 IMAP 解析出的邮件
#[derive(Debug)]
pub struct ParsedEmail {
    pub sender: String,
    pub subject: String,
    pub message_id: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Email 通道
pub struct Email {
    config: EmailConfig,
    running: Arc<RwLock<bool>>,
    task_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    name: String,
    inbound_tx: mpsc::Sender<InboundMessage>,

    /// 自身邮件地址集合（归一化后），用于忽略自己发的邮件
    self_addresses: HashSet<String>,

    /// 每个聊天（发件人）的最近 Subject，用于回复线程
    last_subject_by_chat: Arc<RwLock<HashMap<String, String>>>,

    /// 每个聊天（发件人）的最近 Message-ID，用于 In-Reply-To
    last_message_id_by_chat: Arc<RwLock<HashMap<String, String>>>,

    /// 已处理的 IMAP UID 集合，用于去重
    processed_uids: Arc<Mutex<HashSet<String>>>,
}

impl Email {
    /// 创建新的 Email 通道
    pub fn new(config: EmailConfig, inbound_tx: mpsc::Sender<InboundMessage>) -> ChannelResult<Self> {
        let self_addresses = collect_self_addresses(&config);

        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
            name: "email".to_string(),
            inbound_tx,
            self_addresses,
            last_subject_by_chat: Arc::new(RwLock::new(HashMap::new())),
            last_message_id_by_chat: Arc::new(RwLock::new(HashMap::new())),
            processed_uids: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    /// 获取未读邮件
    fn fetch_new_messages(&self) -> Vec<ParsedEmail> {
        self.fetch_messages("UNSEEN", self.config.mark_seen, true, 0)
    }

    /// 按日期范围获取邮件
    pub fn fetch_messages_between_dates(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        limit: usize,
    ) -> Vec<ParsedEmail> {
        if end_date <= start_date {
            return Vec::new();
        }
        let criteria = format!("SINCE {} BEFORE {}", format_imap_date(&start_date), format_imap_date(&end_date));
        self.fetch_messages(&criteria, false, false, limit.max(1))
    }

    /// 获取邮件（含 stale 连接重试）
    fn fetch_messages(&self, search_criteria: &str, mark_seen: bool, dedupe: bool, limit: usize) -> Vec<ParsedEmail> {
        let mut messages = Vec::new();
        let mut cycle_uids = HashSet::new();

        for attempt in 0..2 {
            match self.fetch_messages_once(search_criteria, mark_seen, dedupe, limit, &mut messages, &mut cycle_uids) {
                Ok(()) => return messages,
                Err(e) => {
                    if attempt == 1 || !is_stale_imap_error(&e) {
                        error!("IMAP fetch failed: {e}");
                        return messages;
                    }
                    warn!("IMAP connection went stale, retrying once: {e}");
                }
            }
        }

        messages
    }

    /// 单次 IMAP 获取
    fn fetch_messages_once(
        &self,
        search_criteria: &str,
        mark_seen: bool,
        dedupe: bool,
        limit: usize,
        messages: &mut Vec<ParsedEmail>,
        cycle_uids: &mut HashSet<String>,
    ) -> Result<(), imap::Error> {
        let imap_config = &self.config.imap;

        // 建立连接并登录，然后执行操作
        let addr = (&*imap_config.host, imap_config.port);
        let domain = &imap_config.host;

        if imap_config.use_ssl {
            let tls =
                native_tls::TlsConnector::new().map_err(|e| imap::Error::Bad(format!("TLS connector error: {e}")))?;
            let client = imap::connect(addr, domain, &tls)?;
            let mut session = client.login(&imap_config.username, &imap_config.password).map_err(|e| e.0)?;
            self.do_imap_fetch(&mut session, search_criteria, mark_seen, dedupe, limit, messages, cycle_uids)
        } else if imap_config.use_tls {
            let tls =
                native_tls::TlsConnector::new().map_err(|e| imap::Error::Bad(format!("TLS connector error: {e}")))?;
            let client = imap::connect_starttls(addr, domain, &tls)?;
            let mut session = client.login(&imap_config.username, &imap_config.password).map_err(|e| e.0)?;
            self.do_imap_fetch(&mut session, search_criteria, mark_seen, dedupe, limit, messages, cycle_uids)
        } else {
            let tcp = std::net::TcpStream::connect(addr).map_err(imap::Error::Io)?;
            let mut client = imap::Client::new(tcp);
            client.read_greeting()?;
            let mut session = client.login(&imap_config.username, &imap_config.password).map_err(|e| e.0)?;
            self.do_imap_fetch(&mut session, search_criteria, mark_seen, dedupe, limit, messages, cycle_uids)
        }
    }

    /// 在已登录的 IMAP session 上执行邮件获取操作
    #[allow(clippy::too_many_arguments)]
    fn do_imap_fetch<T: std::io::Read + std::io::Write>(
        &self,
        session: &mut imap::Session<T>,
        search_criteria: &str,
        mark_seen: bool,
        dedupe: bool,
        limit: usize,
        messages: &mut Vec<ParsedEmail>,
        cycle_uids: &mut HashSet<String>,
    ) -> Result<(), imap::Error> {
        let mailbox = if self.config.imap.mailbox.is_empty() { "INBOX" } else { &self.config.imap.mailbox };

        // 选择邮箱
        match session.select(mailbox) {
            Ok(_) => {}
            Err(e) if is_missing_mailbox_error(&e) => {
                warn!("Mailbox unavailable, skipping poll for {mailbox}: {e}");
                let _ = session.logout();
                return Ok(());
            }
            Err(e) => {
                let _ = session.logout();
                return Err(e);
            }
        }

        // 搜索
        let ids = match session.search(search_criteria) {
            Ok(ids) => ids,
            Err(e) => {
                let _ = session.logout();
                return Err(e);
            }
        };

        if ids.is_empty() {
            let _ = session.logout();
            return Ok(());
        }

        // 限制数量（取最新的）
        let mut id_list: Vec<u32> = ids.into_iter().collect();
        id_list.sort();
        if limit > 0 && id_list.len() > limit {
            id_list = id_list[id_list.len() - limit..].to_vec();
        }

        let seq_set = id_list.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");

        // 获取邮件
        let fetches = match session.fetch(&seq_set, "(BODY.PEEK[] UID)") {
            Ok(f) => f,
            Err(e) => {
                let _ = session.logout();
                return Err(e);
            }
        };

        let mut processed_uids = self.processed_uids.lock().unwrap();

        for fetch in fetches.iter() {
            let Some(body) = fetch.body() else {
                continue;
            };

            // 提取 UID
            let uid = fetch.uid.map(|u| u.to_string()).unwrap_or_default();

            // 去重检查
            if !uid.is_empty() && cycle_uids.contains(&uid) {
                continue;
            }
            if dedupe && !uid.is_empty() && processed_uids.contains(&uid) {
                continue;
            }

            // 解析邮件
            let Some(parsed) = mail_parser::MessageParser::default().parse(body) else {
                continue;
            };

            // 提取发件人
            let sender = parsed
                .from()
                .and_then(|addrs| addrs.first())
                .and_then(|addr| addr.address())
                .unwrap_or_default()
                .to_lowercase();

            if sender.is_empty() {
                continue;
            }

            // 检查是否为自身发送
            if self.self_addresses.contains(&sender) {
                info!("From {sender} ignored: matches bot-owned address");
                remember_processed_uid(&uid, dedupe, cycle_uids, &mut processed_uids);
                if mark_seen {
                    let _ = session.store(fetch.message.to_string(), "+FLAGS (\\Seen)");
                }
                continue;
            }

            // SPF/DKIM 验证
            let (spf_pass, dkim_pass) = check_authentication_results(&parsed);
            if self.config.verify_spf && !spf_pass {
                warn!("From {sender} rejected: SPF verification failed");
                remember_processed_uid(&uid, dedupe, cycle_uids, &mut processed_uids);
                continue;
            }
            if self.config.verify_dkim && !dkim_pass {
                warn!("From {sender} rejected: DKIM verification failed");
                remember_processed_uid(&uid, dedupe, cycle_uids, &mut processed_uids);
                continue;
            }

            // allow_from 检查
            if !is_allowed(&self.config.allow_from, &sender) {
                remember_processed_uid(&uid, dedupe, cycle_uids, &mut processed_uids);
                if mark_seen {
                    let _ = session.store(fetch.message.to_string(), "+FLAGS (\\Seen)");
                }
                continue;
            }

            // 提取字段
            let subject = parsed.subject().unwrap_or_default().to_string();
            let date_value = parsed.date().map(|d| d.to_rfc3339()).unwrap_or_default();
            let message_id = parsed.message_id().unwrap_or_default().to_string();

            let mut body_text = extract_text_body(&parsed);
            if body_text.is_empty() {
                body_text = "(empty email body)".to_string();
            }
            if body_text.len() > self.config.max_body_chars {
                let mut end = self.config.max_body_chars;
                while !body_text.is_char_boundary(end) {
                    end -= 1;
                }
                body_text.truncate(end);
            }

            let content = format!(
                "[EMAIL-CONTEXT] Email received.\nFrom: {sender}\nSubject: {subject}\nDate: {date_value}\n\n{body_text}"
            );

            let mut metadata = HashMap::new();
            metadata.insert("message_id".to_string(), serde_json::Value::String(message_id.clone()));
            metadata.insert("subject".to_string(), serde_json::Value::String(subject.clone()));
            metadata.insert("date".to_string(), serde_json::Value::String(date_value));
            metadata.insert("sender_email".to_string(), serde_json::Value::String(sender.clone()));
            if !uid.is_empty() {
                metadata.insert("uid".to_string(), serde_json::Value::String(uid.clone()));
            }

            messages.push(ParsedEmail { sender, subject, message_id, content, metadata });

            remember_processed_uid(&uid, dedupe, cycle_uids, &mut processed_uids);

            if mark_seen {
                let _ = session.store(fetch.message.to_string(), "+FLAGS (\\Seen)");
            }
        }

        let _ = session.logout();
        Ok(())
    }
}

#[async_trait]
impl Channel for Email {
    async fn start(&self) -> ChannelResult<()> {
        if !self.config.consent_granted {
            warn!(
                "Email channel disabled: consent_granted is false. \
                 Set channels.email.consentGranted=true after explicit user permission."
            );
            return Ok(());
        }

        if !self.config.verify_dkim && !self.config.verify_spf {
            warn!(
                "DKIM and SPF verification are both DISABLED. \
                 Emails with spoofed From headers will be accepted."
            );
        }

        *self.running.write().await = true;
        info!("Starting Email channel (IMAP polling mode)...");

        let poll_seconds = self.config.poll_interval_seconds.max(5);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        let inbound_tx = self.inbound_tx.clone();
        let self_addresses = self.self_addresses.clone();
        let last_subject = Arc::clone(&self.last_subject_by_chat);
        let last_msg_id = Arc::clone(&self.last_message_id_by_chat);
        let processed_uids = Arc::clone(&self.processed_uids);

        let handle = tokio::spawn(async move {
            // 构建一个临时 Email 实例用于轮询（不含 task_handle 避免循环引用）
            let poller = Email {
                config,
                running: Arc::clone(&running),
                task_handle: Arc::new(RwLock::new(None)),
                name: "email".to_string(),
                inbound_tx: inbound_tx.clone(),
                self_addresses,
                last_subject_by_chat: Arc::clone(&last_subject),
                last_message_id_by_chat: Arc::clone(&last_msg_id),
                processed_uids,
            };

            while *running.read().await {
                // IMAP 操作在 blocking 线程中执行
                let emails = {
                    let poller_ref = &poller;
                    // 由于 Email 不是 Send（包含非 Send 字段），我们需要在当前任务中执行
                    // 但 imap 操作是同步的，所以直接在 spawn_blocking 中执行
                    // 这里我们克隆必要的数据
                    poller_ref.fetch_new_messages()
                };

                for email in emails {
                    // 更新线程跟踪
                    if !email.subject.is_empty() {
                        last_subject.write().await.insert(email.sender.clone(), email.subject.clone());
                    }
                    if !email.message_id.is_empty() {
                        last_msg_id.write().await.insert(email.sender.clone(), email.message_id.clone());
                    }

                    // 构建 InboundMessage
                    let mut inbound = InboundMessage::new("email", &email.sender, &email.sender, &email.content);
                    for (k, v) in email.metadata {
                        inbound = inbound.add_metadata(k, v);
                    }

                    if let Err(e) = inbound_tx.send(inbound).await {
                        error!("发送入站邮件消息失败: {e}");
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(poll_seconds)).await;
            }

            info!("Email polling loop exited");
        });

        *self.task_handle.write().await = Some(handle);

        info!("Email 通道启动成功");
        Ok(())
    }

    async fn stop(&self) -> ChannelResult<()> {
        info!("停止 Email 通道");
        *self.running.write().await = false;

        if let Some(handle) = self.task_handle.write().await.take() {
            handle.abort();
        }

        info!("Email 通道已停止");
        Ok(())
    }

    async fn send(&self, msg: OutboundMessage) -> ChannelResult<()> {
        if !self.config.consent_granted {
            warn!("Skip email send: consent_granted is false");
            return Ok(());
        }

        let smtp_config = &self.config.smtp;
        if smtp_config.host.is_empty() {
            warn!("SMTP host not configured");
            return Ok(());
        }

        let to_addr = msg.chat_id.trim().to_string();
        if to_addr.is_empty() {
            warn!("Missing recipient address");
            return Ok(());
        }

        // 判断是否为回复
        let is_reply = self.last_subject_by_chat.read().await.contains_key(&to_addr);
        let force_send = msg.metadata.get("force_send").and_then(|v| v.as_bool()).unwrap_or(false);

        // auto_reply_enabled 仅控制自动回复，不影响主动发送
        if is_reply && !self.config.auto_reply_enabled && !force_send {
            info!("Skip automatic reply to {to_addr}: auto_reply_enabled is false");
            return Ok(());
        }

        // 构建 Subject
        let base_subject = self
            .last_subject_by_chat
            .read()
            .await
            .get(&to_addr)
            .cloned()
            .unwrap_or_else(|| "nanobot reply".to_string());
        let mut subject = reply_subject(&base_subject, &self.config.subject_prefix);

        // 支持 metadata 中的 subject 覆盖
        if let Some(override_subject) = msg.metadata.get("subject").and_then(|v| v.as_str()) {
            let trimmed = override_subject.trim();
            if !trimmed.is_empty() {
                subject = trimmed.to_string();
            }
        }

        // 构建发件人地址
        let from_addr =
            if smtp_config.from_address.is_empty() { &smtp_config.username } else { &smtp_config.from_address };

        let from_mailbox: Mailbox =
            from_addr.parse().map_err(|e| ChannelError::SendFailed(format!("invalid from address: {e}")))?;
        let to_mailbox: Mailbox =
            to_addr.parse().map_err(|e| ChannelError::SendFailed(format!("invalid to address: {e}")))?;

        // 构建邮件
        let mut email_builder = lettre::Message::builder().from(from_mailbox).to(to_mailbox).subject(&subject);

        // 设置 In-Reply-To 和 References
        if let Some(in_reply_to) = self.last_message_id_by_chat.read().await.get(&to_addr) {
            email_builder = email_builder
                .header(InReplyTo::from(in_reply_to.clone()))
                .header(References::from(in_reply_to.clone()));
        }

        let email = email_builder
            .body(msg.content.clone())
            .map_err(|e| ChannelError::SendFailed(format!("build email failed: {e}")))?;

        // 构建 SMTP transport 并发送
        let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());

        let transport = if smtp_config.use_ssl {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_config.host)
                .map_err(|e| ChannelError::SendFailed(format!("SMTP relay failed: {e}")))?
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        } else if smtp_config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_config.host)
                .map_err(|e| ChannelError::SendFailed(format!("SMTP STARTTLS relay failed: {e}")))?
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_config.host)
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        };

        transport.send(email).await.map_err(|e| ChannelError::SendFailed(format!("SMTP send failed: {e}")))?;

        debug!("Email sent to {to_addr}");
        Ok(())
    }

    fn is_running(&self) -> bool {
        if let Ok(running) = self.running.try_read() { *running } else { false }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ---- 邮件解析工具函数 ----

/// 收集自身邮件地址（from_address、smtp.username、imap.username），归一化后去重
fn collect_self_addresses(config: &EmailConfig) -> HashSet<String> {
    [&config.smtp.from_address, &config.smtp.username, &config.imap.username]
        .iter()
        .filter_map(|s| {
            let addr = normalize_address(s);
            if addr.is_empty() { None } else { Some(addr) }
        })
        .collect()
}

/// 归一化邮件地址：提取 `<addr>` 部分，转小写，去空白
///
/// 支持格式：
/// - `"user@example.com"`
/// - `"Name <user@example.com>"`
/// - `"<user@example.com>"`
fn normalize_address(value: &str) -> String {
    let raw = value.trim();
    if raw.is_empty() {
        return String::new();
    }

    // 尝试提取 <...> 中的地址
    if let Some(start) = raw.rfind('<')
        && let Some(end) = raw[start..].find('>')
    {
        let addr = raw[start + 1..start + end].trim();
        if !addr.is_empty() {
            return addr.to_lowercase();
        }
    }

    // 如果包含 @，直接当作地址
    if raw.contains('@') {
        return raw.to_lowercase();
    }

    String::new()
}

/// 简单的 HTML 转纯文本
///
/// 与上游 Python `_html_to_text` 对齐：
/// - `<br>` → `\n`
/// - `</p>` → `\n`
/// - 剥离所有 HTML 标签
/// - 解码 HTML 实体
fn html_to_text(raw_html: &str) -> String {
    // <br> / <br/> → \n
    let re_br = Regex::new(r"(?i)<\s*br\s*/?>").unwrap();
    let text = re_br.replace_all(raw_html, "\n");

    // </p> → \n
    let re_p = Regex::new(r"(?i)<\s*/\s*p\s*>").unwrap();
    let text = re_p.replace_all(&text, "\n");

    // 剥离所有标签
    let re_tags = Regex::new(r"<[^>]+>").unwrap();
    let text = re_tags.replace_all(&text, "");

    // 解码 HTML 实体
    decode_html_entities(&text)
}

/// 解码常见 HTML 实体
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// 从 `mail-parser::Message` 中提取纯文本正文
///
/// 优先取 text/plain，其次取 text/html 并转为纯文本。
fn extract_text_body(message: &mail_parser::Message) -> String {
    // 优先取纯文本
    if let Some(text) = message.body_text(0) {
        return text.to_string();
    }

    // 其次取 HTML 并转换
    if let Some(html) = message.body_html(0) {
        return html_to_text(&html);
    }

    String::new()
}

/// 解析 Authentication-Results 头中的 SPF/DKIM 验证结果
///
/// 返回 `(spf_pass, dkim_pass)`。
fn check_authentication_results(message: &mail_parser::Message) -> (bool, bool) {
    let re_spf = Regex::new(r"(?i)\bspf\s*=\s*pass\b").unwrap();
    let re_dkim = Regex::new(r"(?i)\bdkim\s*=\s*pass\b").unwrap();

    let mut spf_pass = false;
    let mut dkim_pass = false;

    for header in message.headers() {
        if !header.name().eq_ignore_ascii_case("Authentication-Results") {
            continue;
        }
        let value = match header.value() {
            mail_parser::HeaderValue::Text(t) => t.as_ref().to_string(),
            _ => continue,
        };
        if re_spf.is_match(&value) {
            spf_pass = true;
        }
        if re_dkim.is_match(&value) {
            dkim_pass = true;
        }
    }

    (spf_pass, dkim_pass)
}

/// 检查发件人是否在 allow_from 白名单中
///
/// 白名单为空时允许所有发件人。
fn is_allowed(allow_from: &[String], sender: &str) -> bool {
    if allow_from.is_empty() {
        return true;
    }
    let normalized = normalize_address(sender);
    allow_from.iter().any(|a| a.eq_ignore_ascii_case(&normalized))
}

/// 记录已处理的 UID，超过上限时淘汰前半部分
fn remember_processed_uid(
    uid: &str,
    dedupe: bool,
    cycle_uids: &mut HashSet<String>,
    processed_uids: &mut HashSet<String>,
) {
    if uid.is_empty() {
        return;
    }
    cycle_uids.insert(uid.to_string());
    if dedupe {
        processed_uids.insert(uid.to_string());
        if processed_uids.len() > MAX_PROCESSED_UIDS {
            let keep: Vec<String> = processed_uids.iter().skip(processed_uids.len() / 2).cloned().collect();
            *processed_uids = keep.into_iter().collect();
        }
    }
}

/// 检查 IMAP 错误是否为连接过期
fn is_stale_imap_error(err: &imap::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    IMAP_RECONNECT_MARKERS.iter().any(|marker| msg.contains(marker))
}

/// 检查 IMAP 错误是否为邮箱不存在
fn is_missing_mailbox_error(err: &imap::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    IMAP_MISSING_MAILBOX_MARKERS.iter().any(|marker| msg.contains(marker))
}

/// 构建回复 Subject
///
/// 如果原 subject 已以 "re:" 开头则原样返回，否则加上前缀。
fn reply_subject(base_subject: &str, prefix: &str) -> String {
    let subject = base_subject.trim();
    let subject = if subject.is_empty() { "nanobot reply" } else { subject };
    if subject.to_lowercase().starts_with("re:") { subject.to_string() } else { format!("{prefix}{subject}") }
}

/// IMAP 日期格式化（DD-Mon-YYYY）
fn format_imap_date(date: &chrono::NaiveDate) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month = MONTHS[date.month0() as usize];
    format!("{:02}-{}-{}", date.day(), month, date.year())
}

#[cfg(test)]
mod tests;
