# session crate

会话持久化，以 JSONL 格式存储对话历史。

## 架构

```
┌──────────────────────────────────────────┐
│           SessionManager                 │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ 内存缓存: RwLock<HashMap<key, Session>>│  │
│  └──────────────────┬─────────────────┘  │
│                     │                    │
│  get_or_create(key) │                    │
│    1. 查缓存 → 命中返回                   │
│    2. 读磁盘 → 存入缓存                   │
│    3. 都没有 → 新建                       │
│                     │                    │
│  ┌──────────────────┴─────────────────┐  │
│  │ 磁盘: sessions/{key}.jsonl         │  │
│  │  第1行: metadata (timestamps, ...)  │  │
│  │  后续行: 每行一条 Message JSON      │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘

┌────────────────────────────────┐
│ Session (append-only)          │
│  save_turn(): 截断工具结果,     │
│               剥离 base64 图片 │
│  get_history(): 只返回         │
│    last_consolidated 之后的消息 │
└────────────────────────────────┘
```

## 关键类型

- **`Session`** -- `key`, `messages: Vec<Message>`, `created_at`, `updated_at`, `metadata`, `last_consolidated`
  - `add_message(msg)` -- 追加消息
  - `get_history(max_messages, max_tokens, buf) -> usize` -- 将未整合的消息追加到 buf
  - `save_turn(messages, skip)` -- 追加一轮对话的消息（截断工具结果）
  - `clear()` -- 清空消息
- **`SessionManager`** -- 管理 sessions 目录和内存 `RwLock<HashMap>` 缓存
  - `new(workspace)` -- 创建 sessions/ 目录
  - `get_or_create(key) -> Session` -- 从缓存/磁盘加载或新建
  - `save(session)` -- 持久化到 JSONL 并更新缓存
  - `list_sessions() -> Vec<SessionInfo>` -- 列出所有会话
  - `invalidate(key)` -- 从缓存中移除
- **`SessionInfo`** -- `key`, `created_at`, `updated_at`, `path`
- **`SessionMetadata`** -- JSONL 元数据行格式

## 内部依赖

provider, utils
