# provider crate

LLM 提供者抽象层，支持 OpenAI 兼容和 Anthropic Messages API。

## 架构

```
┌─────────────────────────────────┐
│         Provider trait          │
│   chat() + bind_tools()        │
│   (Send + Sync + Clone)        │
└──────────┬──────────┬───────────┘
           │          │
   ┌───────┴───┐  ┌───┴────────────┐
   │OpenAILike │  │AnthropicLike   │
   │async-openai│  │原生 HTTP       │
   └───────┬───┘  └───┬────────────┘
           │          │
   ┌───────┴──────────┴───────────┐
   │      AnyProvider enum        │
   │  from_config() 按配置选择     │
   └──────────────────────────────┘
```

## 关键类型

- **`Provider`** (trait, 要求 `Send + Sync + Clone + 'static`) -- `async fn chat(messages, options) -> Result<Message>` + `fn bind_tools(tools)`
- **`Message`** (enum) -- `System`, `User`, `Assistant`（含 `tool_calls`, `thinking`）, `Tool`；工厂方法 `::system()`, `::user()`, `::assistant()`, `::tool()`
- **`ToolCall`** -- `id`, `name`, `arguments`；`parse_arguments<T>()`, `preview()`
- **`UserContent`** (enum) -- `Text(String)` | `Parts(Vec<ContentPart>)`
- **`ContentPart`** (enum) -- `Text { text }` | `Image { media_type, data }`
- **`Options`** -- `max_tokens`, `temperature`
- **`ProviderResponse`** -- `content`, `tool_calls`
- **`ProviderError`** (enum) -- `Api`, `Timeout`, `Config`
- **`AnyProvider`** (enum) -- `OpenAI(OpenAILike)` | `Anthropic(AnthropicLike)`；`from_config(config) -> Result<Self>`
- **`OpenAILike`** -- 基于 `async-openai` 的 OpenAI 兼容 API 实现
- **`AnthropicLike`** -- 基于原生 HTTP 的 Anthropic Messages API 实现

## 内部依赖

config, tools, utils
