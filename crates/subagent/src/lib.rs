//! Nanobot Subagent - 子代理任务管理器
//!
//! Subagent 组件用于创建和管理后台运行的轻量级代理实例，它们专注于处理特定的后台任务。
//! 子代理与主代理共享相同的 LLM 提供商，但具有独立的上下文和专注的系统提示。
//!
//! # 与 Python 版本的一致性
//!
//! 本实现与 Python 版本的 `SubagentManager` 接口完全一致：
//!
//! - 构造函数参数：`provider`, `workspace`, `bus`, `model`, `temperature`, `max_tokens`, `brave_api_key`, `restrict_to_workspace`
//! - 主方法：`spawn(task, label, origin_channel, origin_chat_id)` - 创建并启动子代理任务
//! - 查询方法：`get_running_count()` - 获取当前运行中的子代理数量
//!
//! # 待实现功能
//!
//! - WebSearchTool 和 WebFetchTool 暂未在 nanobot-tools 中实现，需要后续补充

mod error;
mod manager;
mod task;

pub use error::{SubagentError, SubagentResult};
pub use manager::SubagentManager;
pub use task::Task;
