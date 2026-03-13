//! MCP（Model Context Protocol）服务连接与工具包装器
//!
//! 本模块提供 MCP 客户端功能，允许 AI Agent 通过统一的 Tool trait 接口
//! 调用 MCP 服务器提供的工具。

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http::{HeaderName, HeaderValue};
use nanobot_config::McpServerConfig;
use nanobot_tools::{Tool, ToolContext, ToolError, ToolResult};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ClientCapabilities, ClientInfo, Content, Implementation, RawContent,
    Tool as McpTool,
};
use rmcp::service::{RoleClient, RunningService, serve_client};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{StreamableHttpClientTransport, TokioChildProcess};
use schemars::schema::SchemaObject;
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// MCP 错误类型
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Failed to spawn process: {0}")]
    ProcessSpawnFailed(String),

    #[error("Failed to connect to HTTP server: {0}")]
    HttpConnectionFailed(String),

    #[error("Failed to initialize MCP session: {0}")]
    InitializationFailed(String),

    #[error("Failed to list tools: {0}")]
    ToolListFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// 创建 MCP 客户端信息
fn create_client_info() -> ClientInfo {
    let client_impl = Implementation::new("nanobot", env!("CARGO_PKG_VERSION"));
    ClientInfo::new(ClientCapabilities::default(), client_impl)
}

/// 通过 Stdio 方式连接 MCP 服务器
///
/// # Arguments
/// * `name` - 服务器名称（用于日志）
/// * `command` - 命令路径
/// * `args` - 命令参数
/// * `env` - 环境变量
///
/// # Returns
/// 成功返回 RunningService，失败返回 McpError
async fn connect_stdio(
    name: &str,
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
) -> Result<RunningService<RoleClient, ClientInfo>, McpError> {
    debug!("Connecting to MCP server '{}' via Stdio: {} {:?}", name, command, args);

    // 构建命令
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    // 设置环境变量
    for (key, value) in env {
        cmd.env(key, value);
    }

    // 创建 TokioChildProcess transport
    let mcp_transport = TokioChildProcess::new(cmd).map_err(|e| McpError::ProcessSpawnFailed(e.to_string()))?;

    // 创建客户端信息
    let client_info = create_client_info();

    // 连接并初始化
    serve_client(client_info, mcp_transport)
        .await
        .map_err(|e| McpError::InitializationFailed(e.to_string()))
}

/// 通过 HTTP 方式连接 MCP 服务器
///
/// # Arguments
/// * `name` - 服务器名称（用于日志）
/// * `url` - HTTP 服务器 URL
/// * `headers` - 自定义 HTTP 请求头
///
/// # Returns
/// 成功返回 RunningService，失败返回 McpError
async fn connect_http(
    name: &str,
    url: &str,
    headers: &HashMap<String, String>,
) -> Result<RunningService<RoleClient, ClientInfo>, McpError> {
    debug!(
        "Connecting to MCP server '{}' via HTTP: {} (headers: {})",
        name,
        url,
        headers.len()
    );

    // 创建 HTTP transport 配置
    let mut transport_config = StreamableHttpClientTransportConfig::with_uri(url);

    // 添加自定义 HTTP 头部
    for (key, value) in headers {
        let header_name = HeaderName::try_from(key)
            .map_err(|e| McpError::InvalidConfig(format!("Invalid header name '{}': {}", key, e)))?;
        let header_value = HeaderValue::try_from(value)
            .map_err(|e| McpError::InvalidConfig(format!("Invalid header value for '{}': {}", key, e)))?;
        transport_config.custom_headers.insert(header_name, header_value);
    }

    // 创建 HTTP transport
    let transport = StreamableHttpClientTransport::from_config(transport_config);

    // 创建客户端信息
    let client_info = create_client_info();

    // 连接并初始化
    serve_client(client_info, transport)
        .await
        .map_err(|e| McpError::InitializationFailed(e.to_string()))
}

/// 批量连接多个 MCP 服务器并获取所有工具包装器
///
/// 连接指定的所有 MCP 服务器，获取每个服务器提供的工具，
/// 并将所有工具包装为 nanobot 的 Tool trait 实现。
///
/// # Arguments
/// * `configs` - MCP 服务器配置映射，key 为服务器名称，value 为服务器配置
///
/// # Returns
/// 成功返回所有工具包装器的集合，失败返回 McpError
pub async fn connect(configs: HashMap<String, McpServerConfig>) -> Result<Vec<Box<dyn Tool>>, McpError> {
    let mut all_tools: Vec<Box<dyn Tool>> = Vec::new();

    // 连接每个服务器并获取工具
    for (name, config) in configs {
        debug!("Connecting to MCP server '{}'", name);

        // 根据配置类型连接服务器
        let service = match &config {
            McpServerConfig::Stdio { command, args, env } => connect_stdio(&name, command, args, env).await?,
            McpServerConfig::Http {
                url,
                headers,
                tool_timeout: _,
            } => connect_http(&name, url, headers).await?,
        };

        let service = Arc::new(service);
        let peer = service.peer().clone();

        // 列出工具
        let tools = peer
            .list_tools(Default::default())
            .await
            .map_err(|e| McpError::ToolListFailed(e.to_string()))?;

        let tool_count = tools.tools.len();
        let tool_timeout = config.timeout_duration().as_secs();

        // 为每个工具创建包装器
        for tool in tools.tools {
            let wrapper = McpToolWrapper::new(service.clone(), name.clone(), tool, tool_timeout);
            all_tools.push(Box::new(wrapper));
        }

        info!("Connected to MCP server '{}' and registered {} tools", name, tool_count);
    }

    info!(
        "Successfully connected to all MCP servers, total tools: {}",
        all_tools.len()
    );

    Ok(all_tools)
}

/// MCP 工具包装器
///
/// 将 MCP 服务器提供的工具包装为 nanobot 的 Tool trait 实现，
/// 使得 MCP 工具可以像本地工具一样被调用。
pub struct McpToolWrapper {
    /// MCP 客户端服务（使用 Arc 共享所有权）
    service: Arc<RunningService<RoleClient, ClientInfo>>,
    /// 服务器名称，用于工具名称前缀
    server_name: String,
    /// 原始 MCP 工具定义
    tool_def: McpTool,
    /// 工具调用超时时间
    tool_timeout: Duration,
    /// 包装后的工具名称（mcp_{server_name}_{original_name}）
    wrapped_name: String,
}

impl Drop for McpToolWrapper {
    /// 析构函数：当 Arc 强引用计数为 1 时，关闭 MCP 连接
    fn drop(&mut self) {
        // 检查 Arc 的强引用计数，如果为 1 说明这是最后一个持有者
        if Arc::strong_count(&self.service) == 1 {
            // 获取 cancellation token 并取消连接
            let token = self.service.cancellation_token();
            token.cancel();
            debug!(
                "Closed MCP server connection for '{}' (tool: {})",
                self.server_name, self.wrapped_name
            );
        }
    }
}

impl McpToolWrapper {
    /// 创建 MCP 工具包装器
    ///
    /// # Arguments
    /// * `service` - MCP 客户端服务（使用 Arc 共享）
    /// * `server_name` - MCP 服务器名称
    /// * `tool_def` - MCP 工具定义
    /// * `tool_timeout` - 工具调用超时时间（秒）
    pub fn new(
        service: Arc<RunningService<RoleClient, ClientInfo>>,
        server_name: impl Into<String>,
        tool_def: McpTool,
        tool_timeout: u64,
    ) -> Self {
        let server_name = server_name.into();
        let original_name = tool_def.name.to_string();
        let wrapped_name = format!("mcp_{server_name}_{original_name}");

        Self {
            service,
            server_name,
            tool_def,
            tool_timeout: Duration::from_secs(tool_timeout),
            wrapped_name,
        }
    }

    /// 获取原始工具名称
    pub fn original_name(&self) -> &str {
        &self.tool_def.name
    }

    /// 获取服务器名称
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// 处理工具调用结果，转换为字符串
    fn format_result(result: CallToolResult) -> String {
        if result.content.is_empty() {
            return "(no output)".to_string();
        }

        let parts: Vec<String> = result.content.into_iter().map(Self::content_to_string).collect();

        parts.join("\n")
    }

    /// 将 Content 转换为字符串
    fn content_to_string(content: Content) -> String {
        match content.raw {
            RawContent::Text(text_content) => text_content.text,
            RawContent::Image(image_content) => {
                format!(
                    "[Image: {} ({} bytes)]",
                    image_content.mime_type,
                    image_content.data.len()
                )
            }
            RawContent::Audio(audio_content) => {
                format!(
                    "[Audio: {} ({} bytes)]",
                    audio_content.mime_type,
                    audio_content.data.len()
                )
            }
            RawContent::Resource(embedded_resource) => {
                // EmbeddedResource 是 Annotated<RawEmbeddedResource>，需要访问 .resource 字段
                match &embedded_resource.resource {
                    rmcp::model::ResourceContents::TextResourceContents { text, .. } => text.clone(),
                    rmcp::model::ResourceContents::BlobResourceContents { blob, mime_type, .. } => {
                        let mime = mime_type.as_deref().unwrap_or("unknown");
                        format!("[Blob: {} ({} bytes)]", mime, blob.len())
                    }
                }
            }
            RawContent::ResourceLink(resource) => {
                format!("[Resource: {} ({})]", resource.name, resource.uri)
            }
        }
    }
}

#[async_trait]
impl Tool for McpToolWrapper {
    /// 返回包装后的工具名称
    ///
    /// 格式：mcp_{server_name}_{original_tool_name}
    fn name(&self) -> &str {
        &self.wrapped_name
    }

    /// 返回工具描述
    ///
    /// 如果 MCP 工具没有提供描述，则使用工具名称作为描述
    fn description(&self) -> &str {
        self.tool_def.description.as_deref().unwrap_or(&self.tool_def.name)
    }

    /// 返回工具参数 Schema
    ///
    /// 将 MCP 工具的 inputSchema 转换为 SchemaObject
    fn parameters(&self) -> SchemaObject {
        // 将 JsonObject 转换为 serde_json::Value，然后反序列化为 SchemaObject
        let schema_value = self.tool_def.schema_as_json_value();
        match serde_json::from_value(schema_value) {
            Ok(schema) => schema,
            Err(e) => {
                warn!("Failed to parse tool schema for {}: {}", self.wrapped_name, e);
                // 返回空的 object schema
                SchemaObject::default()
            }
        }
    }

    /// 执行工具调用
    ///
    /// 通过 MCP session 调用原始工具，并处理超时
    async fn execute(&self, _ctx: &ToolContext, params: Value) -> ToolResult {
        // 将参数转换为 JsonObject
        let arguments = if params.is_null() || params.as_object().is_none_or(|m| m.is_empty()) {
            None
        } else {
            match params.as_object() {
                Some(obj) => Some(obj.clone()),
                None => {
                    return Err(ToolError::validation("arguments", "参数必须是 JSON 对象"));
                }
            }
        };

        // 构建调用请求
        let call_params = CallToolRequestParams::new(self.tool_def.name.clone());
        let call_params = if let Some(args) = arguments {
            call_params.with_arguments(args)
        } else {
            call_params
        };

        debug!(
            "Calling MCP tool '{}' on server '{}' with timeout {:?}",
            self.tool_def.name, self.server_name, self.tool_timeout
        );

        // 获取 peer 用于调用工具
        let peer = self.service.peer();

        // 执行调用，带超时控制
        match timeout(self.tool_timeout, peer.call_tool(call_params)).await {
            Ok(Ok(result)) => {
                // 检查是否是错误结果
                if result.is_error.unwrap_or(false) {
                    let output = Self::format_result(result);
                    Err(ToolError::execution(format!("MCP tool returned error: {output}")))
                } else {
                    Ok(Self::format_result(result))
                }
            }
            Ok(Err(e)) => {
                warn!("MCP tool '{}' call failed: {}", self.wrapped_name, e);
                Err(ToolError::execution(format!("MCP tool call failed: {e}")))
            }
            Err(_) => {
                warn!(
                    "MCP tool '{}' timed out after {:?}",
                    self.wrapped_name, self.tool_timeout
                );
                Ok(format!(
                    "(MCP tool call timed out after {}s)",
                    self.tool_timeout.as_secs()
                ))
            }
        }
    }
}
