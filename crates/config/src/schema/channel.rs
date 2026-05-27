//! Channel 配置模块
//!
//! 定义各种通信通道的配置。

use serde::{Deserialize, Serialize};

use super::ConfigError;

/// 钉钉通道配置
///
/// 钉钉通道的配置字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DingTalkConfig {
    /// 是否启用此通道
    #[serde(default)]
    pub enabled: bool,

    /// Client ID (AppKey)
    #[serde(default)]
    pub client_id: String,

    /// Client Secret (AppSecret)
    #[serde(default)]
    pub client_secret: String,

    /// 允许的用户列表（为空则允许所有用户）
    #[serde(default)]
    pub allow_from: Vec<String>,
}

impl DingTalkConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.client_id.is_empty() {
            return Err(ConfigError::Validation("启用的钉钉通道必须配置 client_id".to_string()));
        }
        if self.client_secret.is_empty() {
            return Err(ConfigError::Validation("启用的钉钉通道必须配置 client_secret".to_string()));
        }

        Ok(())
    }
}

/// 飞书通道配置
///
/// 飞书通道的配置字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeishuConfig {
    /// 是否启用此通道
    #[serde(default)]
    pub enabled: bool,

    /// App ID
    #[serde(default)]
    pub app_id: String,

    /// App Secret
    #[serde(default)]
    pub app_secret: String,

    /// 允许的用户列表（为空则允许所有用户）
    #[serde(default)]
    pub allow_from: Vec<String>,

    /// 收到消息时添加的表情回应类型（为空则禁用）
    #[serde(default = "default_react_emoji")]
    pub react_emoji: String,

    /// 是否以引用回复方式发送消息（引用气泡）
    #[serde(default)]
    pub reply_to_message: bool,
}

impl FeishuConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.app_id.is_empty() {
            return Err(ConfigError::Validation("启用的飞书通道必须配置 app_id".to_string()));
        }
        if self.app_secret.is_empty() {
            return Err(ConfigError::Validation("启用的飞书通道必须配置 app_secret".to_string()));
        }

        Ok(())
    }
}

/// 所有通道的配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsConfig {
    /// 钉钉通道配置
    #[serde(default)]
    pub dingtalk: DingTalkConfig,

    /// 飞书通道配置
    #[serde(default)]
    pub feishu: FeishuConfig,

    /// 邮件通道配置
    #[serde(default)]
    pub email: EmailConfig,

    /// 是否发送工具提示（CLI 模式）
    #[serde(default)]
    pub send_tool_hints: bool,

    /// 是否发送进度信息（CLI 模式）
    #[serde(default = "default_send_progress")]
    pub send_progress: bool,
}

/// 邮件通道配置（IMAP 收件 + SMTP 发件）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EmailConfig {
    /// 是否启用此通道
    pub enabled: bool,

    /// 用户显式授权开关（防止误配置导致邮件被读取）
    pub consent_granted: bool,

    /// IMAP 收件配置
    pub imap: ImapConfig,

    /// SMTP 发件配置
    pub smtp: SmtpConfig,

    /// 是否自动回复入站邮件（默认 true）
    pub auto_reply_enabled: bool,

    /// 轮询间隔（秒，默认 30）
    pub poll_interval_seconds: u64,

    /// 是否将已处理邮件标记为已读（默认 true）
    pub mark_seen: bool,

    /// 邮件正文最大字符数（默认 12000）
    pub max_body_chars: usize,

    /// 回复邮件的 Subject 前缀（默认 "Re: "）
    pub subject_prefix: String,

    /// 允许的发件人列表（为空则允许所有）
    pub allow_from: Vec<String>,

    /// 是否验证 DKIM（默认 true）
    pub verify_dkim: bool,

    /// 是否验证 SPF（默认 true）
    pub verify_spf: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            consent_granted: false,
            imap: ImapConfig::default(),
            smtp: SmtpConfig::default(),
            auto_reply_enabled: true,
            poll_interval_seconds: 30,
            mark_seen: true,
            max_body_chars: 12000,
            subject_prefix: "Re: ".to_string(),
            allow_from: Vec::new(),
            verify_dkim: true,
            verify_spf: true,
        }
    }
}

impl EmailConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.imap.validate()?;
        self.smtp.validate()?;
        Ok(())
    }
}

/// IMAP 收件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ImapConfig {
    /// 服务器地址
    pub host: String,

    /// 端口（默认 993）
    pub port: u16,

    /// 用户名
    pub username: String,

    /// 密码（或授权码）
    pub password: String,

    /// 邮箱名称（默认 "INBOX"）
    pub mailbox: String,

    /// 是否使用 SSL 直连（默认 true，端口 993）
    pub use_ssl: bool,

    /// 是否使用 STARTTLS（默认 false，端口 143 先明文再升级）
    pub use_tls: bool,
}

impl Default for ImapConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 993,
            username: String::new(),
            password: String::new(),
            mailbox: "INBOX".to_string(),
            use_ssl: true,
            use_tls: false,
        }
    }
}

impl ImapConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        let required =
            [("imap.host", &self.host), ("imap.username", &self.username), ("imap.password", &self.password)];
        for (name, value) in required {
            if value.is_empty() {
                return Err(ConfigError::Validation(format!("启用的邮件通道必须配置 {name}")));
            }
        }
        Ok(())
    }
}

/// SMTP 发件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SmtpConfig {
    /// 服务器地址
    pub host: String,

    /// 端口（默认 587）
    pub port: u16,

    /// 用户名
    pub username: String,

    /// 密码（或授权码）
    pub password: String,

    /// 发件人地址（为空则使用 username）
    pub from_address: String,

    /// 是否使用 SSL 直连（默认 false，端口 465）
    pub use_ssl: bool,

    /// 是否使用 STARTTLS（默认 true，端口 587）
    pub use_tls: bool,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 587,
            username: String::new(),
            password: String::new(),
            from_address: String::new(),
            use_ssl: false,
            use_tls: true,
        }
    }
}

impl SmtpConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        let required =
            [("smtp.host", &self.host), ("smtp.username", &self.username), ("smtp.password", &self.password)];
        for (name, value) in required {
            if value.is_empty() {
                return Err(ConfigError::Validation(format!("启用的邮件通道必须配置 {name}")));
            }
        }
        Ok(())
    }
}

fn default_send_progress() -> bool {
    true
}

fn default_react_emoji() -> String {
    "THUMBSUP".to_string()
}
