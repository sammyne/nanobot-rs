//! 钉钉媒体发送辅助函数
//!
//! 提供媒体类型识别、文件名猜测、MIME 类型推断、字节读取、上传和 batch send 等功能。

use std::path::Path;

use dingtalk_stream::ChatbotReplier;
use dingtalk_stream::transport::http::HttpClient;
use tracing::warn;

use crate::error::{ChannelError, ChannelResult};

/// 钉钉 batch send API 地址
const BATCH_SEND_URL: &str = "https://api.dingtalk.com/v1.0/robot/oToMessages/batchSend";

const IMAGE_EXTS: &[&str] = &[".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp"];
const AUDIO_EXTS: &[&str] = &[".amr", ".mp3", ".wav", ".ogg", ".m4a", ".aac"];
const VIDEO_EXTS: &[&str] = &[".mp4", ".mov", ".avi", ".mkv", ".webm"];

/// 检查路径是否为 HTTP(S) URL
pub(super) fn is_http_url(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// 根据文件扩展名猜测上传类型
///
/// 返回 `"image"` / `"voice"` / `"video"` / `"file"`。
pub(super) fn guess_upload_type(media_ref: &str) -> &'static str {
    let ext = extract_extension(media_ref);
    if IMAGE_EXTS.contains(&ext.as_str()) {
        "image"
    } else if AUDIO_EXTS.contains(&ext.as_str()) {
        "voice"
    } else if VIDEO_EXTS.contains(&ext.as_str()) {
        "video"
    } else {
        "file"
    }
}

/// 从路径或 URL 提取文件名
///
/// 无法提取时按 `upload_type` 返回 fallback 名称。
pub(super) fn guess_filename(media_ref: &str, upload_type: &str) -> String {
    // 对 URL 取 path 部分，对本地路径直接用
    let path_str = if is_http_url(media_ref) {
        // 取 ? 之前的部分，再取最后一段
        media_ref.split('?').next().unwrap_or(media_ref)
    } else if let Some(stripped) = media_ref.strip_prefix("file://") {
        // file:///path/to/file → /path/to/file
        stripped
    } else {
        media_ref
    };

    let name = Path::new(path_str).file_name().and_then(|n| n.to_str()).unwrap_or("");

    if name.is_empty() {
        match upload_type {
            "image" => "image.jpg".to_owned(),
            "voice" => "audio.amr".to_owned(),
            "video" => "video.mp4".to_owned(),
            _ => "file.bin".to_owned(),
        }
    } else {
        name.to_owned()
    }
}

/// 根据文件名扩展名猜测 MIME 类型
pub(super) fn guess_mime_type(filename: &str) -> &'static str {
    let ext = extract_extension(filename);
    match ext.as_str() {
        // 图片
        ".jpg" | ".jpeg" => "image/jpeg",
        ".png" => "image/png",
        ".gif" => "image/gif",
        ".bmp" => "image/bmp",
        ".webp" => "image/webp",
        // 音频
        ".amr" => "audio/amr",
        ".mp3" => "audio/mpeg",
        ".wav" => "audio/wav",
        ".ogg" => "audio/ogg",
        ".m4a" => "audio/mp4",
        ".aac" => "audio/aac",
        // 视频
        ".mp4" => "video/mp4",
        ".mov" => "video/quicktime",
        ".avi" => "video/x-msvideo",
        ".mkv" => "video/x-matroska",
        ".webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

/// 从 HTTP URL 下载或从本地文件读取媒体字节
///
/// 返回 `(字节, 文件名, MIME 类型)`。
pub(super) async fn read_media_bytes(media_ref: &str) -> ChannelResult<(Vec<u8>, String, String)> {
    let upload_type = guess_upload_type(media_ref);

    if is_http_url(media_ref) {
        let resp = reqwest::get(media_ref).await.map_err(|e| ChannelError::SendFailed(format!("媒体下载失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(ChannelError::SendFailed(format!("媒体下载失败 status={}", resp.status())));
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.split(';').next().unwrap_or(ct).trim().to_owned());

        let bytes = resp.bytes().await.map_err(|e| ChannelError::SendFailed(format!("读取媒体字节失败: {e}")))?;

        let filename = guess_filename(media_ref, upload_type);
        let mime = content_type.unwrap_or_else(|| guess_mime_type(&filename).to_owned());

        return Ok((bytes.to_vec(), filename, mime));
    }

    // 本地文件
    let local_path = if let Some(stripped) = media_ref.strip_prefix("file://") {
        std::path::PathBuf::from(stripped)
    } else {
        std::path::PathBuf::from(media_ref)
    };

    let bytes = tokio::fs::read(&local_path)
        .await
        .map_err(|e| ChannelError::SendFailed(format!("读取本地文件失败 {}: {e}", local_path.display())))?;

    let filename = local_path.file_name().and_then(|n| n.to_str()).unwrap_or("file.bin").to_owned();
    let mime = guess_mime_type(&filename).to_owned();

    Ok((bytes, filename, mime))
}

/// 通过 `oToMessages/batchSend` API 发送消息
pub(super) async fn send_batch_message(
    http_client: &HttpClient,
    token: &str,
    robot_code: &str,
    staff_id: &str,
    msg_key: &str,
    msg_param: &serde_json::Value,
) -> ChannelResult<()> {
    let body = serde_json::json!({
        "robotCode": robot_code,
        "userIds": [staff_id],
        "msgKey": msg_key,
        "msgParam": serde_json::to_string(msg_param).unwrap_or_default(),
    });

    http_client
        .post_json(BATCH_SEND_URL, &body, Some(token))
        .await
        .map_err(|e| ChannelError::SendFailed(format!("batch send 失败 msgKey={msg_key}: {e}")))?;

    Ok(())
}

/// 发送单个媒体文件，含 fallback 链
///
/// 流程：
/// 1. HTTP URL 图片 → `sampleImageMsg` + `photoURL`
/// 2. 失败 → 读取字节 → upload → `sampleImageMsg` + `mediaId`（图片）
/// 3. 非图片或图片 mediaId 失败 → `sampleFile` + `mediaId`
pub(super) async fn send_media_ref(
    replier: &ChatbotReplier,
    http_client: &HttpClient,
    token: &str,
    robot_code: &str,
    staff_id: &str,
    media_ref: &str,
) -> ChannelResult<()> {
    let media_ref = media_ref.trim();
    if media_ref.is_empty() {
        return Ok(());
    }

    let upload_type = guess_upload_type(media_ref);

    // 快速路径：HTTP URL 图片直接发
    if upload_type == "image" && is_http_url(media_ref) {
        let param = serde_json::json!({"photoURL": media_ref});
        if send_batch_message(http_client, token, robot_code, staff_id, "sampleImageMsg", &param).await.is_ok() {
            return Ok(());
        }
        warn!("钉钉图片 URL 直接发送失败，尝试 upload fallback: {media_ref}");
    }

    // 读取字节 → 上传
    let (bytes, filename, mime) = read_media_bytes(media_ref).await?;

    let media_id = replier
        .upload_to_dingtalk(&bytes, upload_type, &filename, &mime)
        .await
        .map_err(|e| ChannelError::SendFailed(format!("媒体上传失败: {e}")))?;

    // 图片：尝试 sampleImageMsg + mediaId
    if upload_type == "image" {
        let param = serde_json::json!({"photoURL": media_id});
        if send_batch_message(http_client, token, robot_code, staff_id, "sampleImageMsg", &param).await.is_ok() {
            return Ok(());
        }
        warn!("钉钉图片 mediaId 发送失败，fallback 到 sampleFile: {media_ref}");
    }

    // 最终 fallback：sampleFile
    let file_ext = extract_extension(&filename);
    let file_type = file_ext.trim_start_matches('.');
    let file_type = if file_type == "jpeg" { "jpg" } else { file_type };

    let param = serde_json::json!({
        "mediaId": media_id,
        "fileName": filename,
        "fileType": file_type,
    });
    send_batch_message(http_client, token, robot_code, staff_id, "sampleFile", &param).await
}

/// 从路径或 URL 提取小写扩展名（含 `.`），如 `".jpg"`
fn extract_extension(media_ref: &str) -> String {
    // 对 URL 去掉查询参数
    let path_part = if is_http_url(media_ref) { media_ref.split('?').next().unwrap_or(media_ref) } else { media_ref };

    Path::new(path_part)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()))
        .unwrap_or_default()
}
