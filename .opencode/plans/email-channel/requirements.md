# 需求

## 目标与背景

为 nanobot-rs 实现 email 通道，对齐上游 HKUDS/nanobot 的 `EmailChannel`（Python，~683 行）。email 通道通过 IMAP 轮询接收邮件、SMTP 发送回复，使 agent 能够以邮件为交互界面。

当前 nanobot-rs 已有 DingTalk 和 Feishu 两个通道实现，Channel trait、ChannelManager、InboundMessage/OutboundMessage、ChannelError 等基础设施完备。新增 email 通道只需实现 Channel trait 并在 ChannelManager 和 config 中注册。

不做此需求的影响：无法通过邮件与 agent 交互，缺少上游已有的通道能力。

## 方案比较（强制）

### 方案 1: 同步 IMAP + spawn_blocking（最小可行版）

- 思路: 使用同步 `imap` crate 进行 IMAP 操作，通过 `tokio::task::spawn_blocking` 包装为异步。SMTP 使用 `lettre` crate（支持 tokio async transport）。邮件解析使用 `mail-parser` crate。与上游 Python 实现（`imaplib` + `asyncio.to_thread`）模式一致。
- 优点: 同步 IMAP 库成熟稳定；与上游实现模式对齐，移植逻辑直观；`lettre` 是 Rust 生态中最成熟的邮件发送库
- 缺点: `spawn_blocking` 占用线程池线程，高并发场景下有开销（但邮件轮询频率低，实际无影响）
- 工作量估算: M

### 方案 2: 全异步 async-imap（理想架构）

- 思路: 使用 `async-imap` crate 进行异步 IMAP 操作，避免 `spawn_blocking`。SMTP 和邮件解析同方案 1。
- 优点: 纯异步，不占用 blocking 线程池
- 缺点: `async-imap` 维护活跃度不如同步 `imap`；API 差异需要额外适配工作；邮件轮询本身是低频操作（30s 间隔），异步优势不明显
- 工作量估算: M

### 推荐

推荐方案 1。理由：邮件轮询是低频操作（默认 30s 间隔），`spawn_blocking` 的开销可忽略；同步 `imap` 库更成熟；与上游 Python 实现模式一致，移植逻辑更直观。

## 功能需求列表

### 核心功能

1. **IMAP 轮询收件** -- 定时轮询 IMAP 邮箱获取未读邮件，解析为 InboundMessage 发送到 agent
2. **SMTP 发送回复** -- 接收 OutboundMessage，通过 SMTP 发送邮件回复
3. **邮件正文解析** -- 提取纯文本正文；HTML 邮件转为纯文本；多部分邮件优先取 text/plain
4. **SPF/DKIM 验证** -- 解析 Authentication-Results 邮件头，拒绝未通过 SPF/DKIM 验证的邮件（可配置开关）
5. **权限控制** -- allow_from 白名单过滤发件人；自身地址检测（忽略自己发的邮件）
6. **UID 去重** -- 基于 IMAP UID 去重，避免重复处理同一封邮件
7. **邮件线程** -- 回复时设置 Subject 前缀（Re:）、In-Reply-To 和 References 头，保持邮件线程
8. **配置集成** -- EmailConfig（含内嵌 ImapConfig/SmtpConfig）加入 ChannelsConfig，ChannelManager 中注册 email 通道。配置结构：`EmailConfig { imap: ImapConfig, smtp: SmtpConfig, ... }`，IMAP 和 SMTP 各自包含 host/port/username/password/use_ssl/use_tls 字段
9. **consent_granted 安全开关** -- 需要用户显式授权才启用邮件通道（防止误配置）

### 扩展功能

10. **IMAP 断线重试** -- 检测 stale 连接错误，自动重试一次
11. **历史邮件查询** -- 按日期范围获取邮件（供 cron/heartbeat 等场景使用）

### 不纳入本次

- **附件处理** -- 按配置的 MIME 类型白名单提取附件，保存到本地目录。留作后续迭代。

## 非功能需求

- **性能**: 轮询间隔可配置（默认 30s），单次轮询在 spawn_blocking 中执行，不阻塞 tokio runtime
- **安全**: consent_granted 默认 false；SPF/DKIM 验证默认开启；SMTP 支持 TLS/SSL
- **兼容性**: 配置字段使用 camelCase（与现有通道一致）；Channel trait 接口不变
- **可维护性**: 遵循项目模块布局规范（`email/mod.rs` + `email/tests.rs`）；错误使用现有 ChannelError 枚举
- **测试要求**: 单元测试覆盖邮件解析、SPF/DKIM 验证、HTML-to-text 转换、allow_from 过滤、自身地址检测、UID 去重逻辑

## 边界与不做事项

- 不实现 IMAP IDLE（推送模式），仅轮询
- 不实现 OAuth2 认证，仅用户名/密码
- 不实现富文本/HTML 格式的出站邮件，仅纯文本
- 不实现附件处理，留作后续迭代

## 假设与约束

- **技术假设**: 目标邮件服务器支持 IMAP4/SMTP 标准协议；Authentication-Results 头由邮件服务器填充
- **资源约束**: 新增 3 个外部依赖（`imap`、`lettre`、`mail-parser`），均为成熟的 Rust crate
- **环境约束**: 需要可访问的 IMAP/SMTP 服务器进行集成测试（单元测试不需要）

## 待确认事项

无
