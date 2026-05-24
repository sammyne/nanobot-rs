# context crate

LLM 上下文构建器，组装系统提示和消息列表。

## 关键类型

- **`ContextBuilder`** -- 持有 workspace 路径、`Arc<MemoryStore>`、`SkillsLoader`
  - `new(workspace)` -- 初始化，规范化工作空间路径，创建 MemoryStore 和 SkillsLoader
  - `memory() -> Arc<MemoryStore>` -- 返回记忆存储的共享引用
  - `build_system_prompt() -> Result<String>` -- 组装：核心身份 + bootstrap 文件 + 记忆 + 活跃技能 + 技能摘要
  - `build_messages(history, current_message, media, channel, chat_id) -> Result<Vec<Message>>` -- 构建完整消息列表
  - `build_core_identity() -> String` -- nanobot 身份信息段
  - `load_bootstrap_files() -> String` -- 加载 AGENTS.md, SOUL.md, USER.md, TOOLS.md, IDENTITY.md
- **`ContextError`** (enum) -- `Io`, `MediaType`, `Memory`

## 内部依赖

provider, memory, skills
