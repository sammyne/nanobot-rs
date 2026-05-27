# heartbeat crate

周期性心跳检查，通过两阶段决策（决策+执行）避免不必要的代理唤醒。执行后可通过评估回调决定是否通知用户。

## 关键类型

- **`HeartbeatService<P: Provider>`** -- 读取 HEARTBEAT.md，LLM 决策是否执行，触发回调
  - `new(workspace, provider, config, on_execute, on_evaluate, on_notify)` -- 创建服务并绑定心跳工具
  - `start(self) -> Result<(), HeartbeatError>` -- 启动周期性心跳循环（阻塞）
- **`HeartbeatError`** (enum) -- `Disabled`, `InvalidConfig`, `FileRead`, `Provider`, `Parse`, `Execute`, `Notify`
- **`OnExecuteCallback`** -- `Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<String, anyhow::Error>> + Send>> + Send + Sync>`
- **`OnEvaluateCallback`** -- `Arc<dyn Fn(&str, &str) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>`（参数：response, task_context）
- **`OnNotifyCallback`** -- `Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> + Send + Sync>`

## 内部依赖

provider, tools, config
