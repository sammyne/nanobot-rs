# 实施计划

- [ ] 1. 定义 GatewayConfig 结构体
   - 在 `crates/config/src/schema/` 目录下创建 `gateway.rs` 文件
   - 定义 `GatewayConfig` 结构体，包含 `host`（String，默认 "0.0.0.0"）和 `port`（u16，默认 18790）字段
   - 实现 `Default` trait，使用 serde 的 `#[serde(default)]` 属性支持字段默认值
   - 在 `mod.rs` 中添加 `pub mod gateway;` 并导出 `GatewayConfig`
   - _需求：1.1、1.2、1.3、1.4、1.5_

- [ ] 2. 将 gateway 字段集成到 Config 结构体
   - 在 `Config` 结构体中添加 `pub gateway: GatewayConfig` 字段
   - 使用 `#[serde(default)]` 属性确保未配置时使用默认值
   - _需求：2.1、2.2、2.3_

- [ ] 3. 实现 GatewayConfig 的 validate 方法
   - 为 `GatewayConfig` 实现 `validate` 方法
   - 验证 `port > 0`，否则返回错误 "gateway.port 必须大于 0"
   - 验证 `host` 非空，否则返回错误 "gateway.host 不能为空"
   - 在 `Config` 的验证方法中调用 `self.gateway.validate()` 进行验证
   - _需求：4.1、4.2、4.3_

- [ ] 4. 修改 gateway 命令使用配置文件参数
   - 修改 `crates/cli/src/commands/gateway/mod.rs` 中的命令实现
   - 从配置文件的 `config.gateway.port` 读取端口作为默认值
   - 当命令行指定 `--port` 时，覆盖配置文件的值
   - 添加日志输出，显示实际使用的端口及来源（配置文件或命令行）
   - _需求：3.1、3.2、3.3、3.4_

- [ ] 5. 更新配置文件示例和测试
   - 在项目中的配置示例文件添加 `gateway` 配置节
   - 编写单元测试验证 `GatewayConfig` 的序列化/反序列化和默认值
   - 编写集成测试验证 gateway 命令的配置加载逻辑
   - _需求：5.1、5.2_
