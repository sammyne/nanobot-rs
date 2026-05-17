# 需求

## 目标与背景

nanobot-rs 目前通过 `OpenAILike` 结构体支持 OpenAI 兼容的 Chat Completions API。所有 LLM 调用都经由 `async-openai` 库发送到 OpenAI 格式的端点。现在需要扩展支持 Anthropic 原生 Messages API 协议，使项目能够直接调用 Anthropic Claude 系列模型，而无需依赖 OpenAI 兼容代理。

当前架构要点：
- `Provider` trait 已定义为通用抽象（`chat()` + `bind_tools()`），所有库 crate 通过 `P: Provider` 泛型使用
- 唯一的具体实现是 `OpenAILike`，仅在二进制 crate (`crates/nanobot/`) 中直接引用
- `Provider` trait 要求 `Clone`，不支持 `dyn Provider` 动态分发
- 配置中只有单一 `ProviderConfig`（`api_key` + `api_base` + `extra_headers`）
- 默认模型已经是 `"anthropic/claude-opus-4-5"`（但当前通过 OpenAI 兼容代理调用）

## 方案比较

### 方案 1: 枚举包装器 + reqwest 直接实现

- 思路: 创建 `AnthropicProvider` 结构体，使用 `reqwest` 直接调用 Anthropic Messages API。引入 `AnyProvider` 枚举包装 `OpenAILike` 和 `AnthropicProvider`，实现 `Provider` trait 的静态分发。通过模型名称前缀自动选择 provider 类型。
- 优点:
  - 完全控制 Anthropic API 交互细节
  - 无需引入第三方 Anthropic 客户端库（`reqwest` 项目中已有使用）
  - 枚举分发是 Rust 惯用模式，零运行时开销
  - 对现有库 crate 零侵入（它们继续使用 `P: Provider` 泛型）
  - 模型名称前缀路由与现有 `"anthropic/claude-opus-4-5"` 命名约定一致
- 缺点:
  - 需要手动实现 Anthropic API 的请求/响应序列化
  - 每新增一个 provider 需要扩展枚举

### 方案 2: 去掉 Clone 约束，改用 `Arc<dyn Provider>`

- 思路: 移除 `Provider` trait 的 `Clone` 约束，改用 `Arc<dyn Provider>` 实现动态分发。各 provider 独立实现 trait。
- 优点:
  - 新增 provider 无需修改枚举
  - 更灵活的运行时多态
- 缺点:
  - 需要修改所有使用 `P: Provider` 泛型的库 crate（`agent`、`subagent`、`heartbeat`、`memory` 等），改为接受 `Arc<dyn Provider>`
  - 破坏性变更范围大，涉及大量文件
  - 动态分发有微小运行时开销
  - 当前 `bind_tools(&mut self)` 需要 `&mut` 访问，与 `Arc` 不兼容，需要引入内部可变性

### 方案 3: 使用第三方 Anthropic Rust 客户端库

- 思路: 使用 `anthropic-api` 等第三方 crate 封装 Anthropic API 调用，类似 `async-openai` 对 OpenAI 的封装。
- 优点:
  - 减少手动序列化代码
- 缺点:
  - 目前 Rust 生态中没有成熟、广泛使用的 Anthropic 客户端库
  - 引入不稳定的第三方依赖增加维护风险
  - 对 API 细节的控制力降低

### 推荐

推荐**方案 1**。理由：
1. 对现有架构侵入最小——所有库 crate 无需修改
2. 枚举分发是 Rust 惯用模式，类型安全且零开销
3. Anthropic Messages API 结构简单，手动实现 reqwest 调用的工作量可控
4. 与现有模型命名约定（`provider/model`）天然契合

## 功能需求列表

### 核心功能

1. **Anthropic Messages API 客户端实现**
   - 实现 `AnthropicLike` 结构体，使用 `reqwest` 调用 Anthropic Messages API (`POST /v1/messages`)
   - 支持 `x-api-key` 和 `anthropic-version` 请求头认证
   - 正确处理 Anthropic 的系统消息格式（独立 `system` 参数，而非消息数组中的 system role）
   - 支持请求超时控制

2. **工具调用（Tool Use）支持**
   - 将内部 `ToolDefinition` 转换为 Anthropic 的 tool 格式（`name`、`description`、`input_schema`）
   - 解析响应中的 `tool_use` content block，转换为内部 `ToolCall`
   - 将工具执行结果作为 `tool_result` content block 发送回 Anthropic API

3. **消息格式转换**
   - `Message::System` → Anthropic `system` 参数（从消息数组中提取）
   - `Message::User` → Anthropic `user` 角色消息
   - `Message::Assistant` → Anthropic `assistant` 角色消息（含可选 `tool_use` content blocks）
   - `Message::Tool` → Anthropic `user` 角色消息中的 `tool_result` content block

4. **Provider 枚举包装器**
   - 创建 `AnyProvider` 枚举，包装 `OpenAILike` 和 `AnthropicLike`
   - 为 `AnyProvider` 实现 `Provider` trait，委托到具体实现
   - 提供 `AnyProvider::from_config(config)` 工厂方法，根据 `ProvidersConfig` 枚举变体选择 provider

5. **配置扩展**
   - 将 `ProvidersConfig` 从 struct 改为枚举，变体为 `Custom(ProviderConfig)` 和 `Anthropic(ProviderConfig)`
   - `AnyProvider::from_config()` 的选择逻辑：match `ProvidersConfig` 变体，`Custom` → `OpenAILike`，`Anthropic` → `AnthropicLike`
   - Anthropic 的默认 `api_base` 为 `"https://api.anthropic.com"`

6. **二进制 crate 适配**
   - 将 `crates/nanobot/` 中的 `OpenAILike` 替换为 `AnyProvider`
   - 涉及 `commands/agent/mod.rs` 和 `commands/gateway/mod.rs` 两个文件

### 扩展功能

- 暂无。流式响应（streaming）不在本次范围内。

## 非功能需求

- **性能**：枚举分发零运行时开销；reqwest 连接池复用
- **安全**：API Key 不在日志中明文输出（复用现有 `masked_api_key` 机制）
- **兼容性**：现有 OpenAI 兼容 API 的使用方式完全不受影响；配置文件向后兼容（`providers.anthropic` 为可选字段）
- **可维护性**：Anthropic provider 代码结构与 OpenAI provider 对称，便于理解和维护
- **测试要求**：
  - 消息格式转换的单元测试（`Message` ↔ Anthropic 请求/响应格式）
  - 工具调用格式转换的单元测试
  - `AnyProvider` 枚举分发的单元测试
  - 配置解析和 provider 选择逻辑的单元测试

## 边界与不做事项

- 不实现流式响应（streaming）
- 不实现 Anthropic 的 vision/image 输入功能
- 不实现 Anthropic 的 prompt caching 功能
- 不修改现有库 crate 的泛型签名（`P: Provider`）
- 不重构现有 `OpenAILike` 实现
- 不支持运行时动态切换 provider（provider 在启动时确定）

## 假设与约束

- **技术假设**：Anthropic Messages API 版本使用 `2023-06-01`（当前稳定版本）
- **资源约束**：`reqwest` 已在项目中使用（`crates/channels/`），可提升为工作空间依赖
- **环境约束**：Anthropic API Key 可通过配置文件或环境变量 `NANOBOT_PROVIDERS__ANTHROPIC__API_KEY` 设置

## 待确认事项

无

## 已确认决策

1. **Anthropic API 版本**：使用 `2023-06-01`（Anthropic 唯一的稳定版本，所有官方 SDK 均硬编码此值）
2. **模型名称路由规则**：仅通过 `"anthropic/"` 前缀判断使用 Anthropic provider，不支持其他模式
3. **extra_headers 支持**：支持。`providers.anthropic` 复用现有 `ProviderConfig` 结构，天然包含 `extra_headers` 字段
4. **配置范围**：仅新增 `providers.anthropic` 字段（最小变更），不预留其他 provider 字段
