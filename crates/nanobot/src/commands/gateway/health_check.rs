//! 健康检查服务模块
//!
//! 提供极简的 HTTP 健康检查端点，用于 Kubernetes、Docker 等容器编排平台的服务探活。
//!
//! # 特点
//!
//! - 基于 TCP 的极简 HTTP 实现，不依赖外部 HTTP 框架
//! - 对所有请求返回 HTTP 200 状态码

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

/// 启动健康检查服务
///
/// 启动一个极简的 HTTP 健康检查服务。
/// 对所有请求返回 HTTP 200 状态码。
/// 如果端口被占用，记录错误日志但不返回错误。
///
/// # 参数
///
/// * `port` - 监听端口
pub async fn serve(port: u16) {
    // 绑定 TCP 监听器
    let listener = match TcpListener::bind(format!("0.0.0.0:{port}")).await {
        Ok(l) => l,
        Err(e) => {
            error!("健康检查服务启动失败: 端口 {} 被占用 ({})", port, e);
            return;
        }
    };

    info!("健康检查服务已启动，监听端口 {}", port);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("健康检查请求来自 {}", addr);
                if let Err(e) = handle_request(stream).await {
                    warn!("处理健康检查请求失败: {}", e);
                }
            }
            Err(e) => {
                warn!("接受连接失败: {}", e);
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

/// 处理单个 HTTP 请求
///
/// 仅解析请求行，忽略所有请求头和请求体，返回 HTTP 200 响应。
async fn handle_request(mut stream: TcpStream) -> std::io::Result<()> {
    // 读取请求数据（仅读取请求行，忽略其余部分）
    let mut buffer = [0u8; 1024];
    let _ = stream.read(&mut buffer).await;

    // 构造 HTTP 200 响应（使用 HTTP/1.0 以确保最佳兼容性）
    let response = "HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}
