use tokio::sync::mpsc;

use super::*;

/// 测试钉钉通道创建
#[tokio::test]
async fn dingtalk_creation() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let dingtalk = DingTalk::new(config, inbound_tx).await;
    assert!(dingtalk.is_ok());
}

/// 测试权限检查功能
#[tokio::test]
async fn permission_check() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: vec!["user1".to_string(), "user2".to_string()],
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let dingtalk = DingTalk::new(config, inbound_tx).await.unwrap();

    assert!(dingtalk.check_permission("user1"));
    assert!(dingtalk.check_permission("user2"));
    assert!(!dingtalk.check_permission("user3"));

    // 测试带分隔符的发送者 ID
    assert!(dingtalk.check_permission("user1|extra"));
    assert!(!dingtalk.check_permission("user3|extra"));
}

// ── media helper tests ──

#[test]
fn is_http_url_variants() {
    assert!(media::is_http_url("http://example.com/img.png"));
    assert!(media::is_http_url("https://example.com/img.png"));
    assert!(media::is_http_url("HTTP://EXAMPLE.COM/IMG.PNG"));
    assert!(media::is_http_url("Https://Mixed.Case"));

    assert!(!media::is_http_url("/tmp/photo.jpg"));
    assert!(!media::is_http_url("file:///tmp/photo.jpg"));
    assert!(!media::is_http_url(""));
    assert!(!media::is_http_url("ftp://example.com/file"));
}

#[test]
fn guess_upload_type_by_extension() {
    // 图片
    assert_eq!(media::guess_upload_type("photo.jpg"), "image");
    assert_eq!(media::guess_upload_type("photo.JPEG"), "image");
    assert_eq!(media::guess_upload_type("photo.png"), "image");
    assert_eq!(media::guess_upload_type("photo.webp"), "image");

    // 音频
    assert_eq!(media::guess_upload_type("audio.mp3"), "voice");
    assert_eq!(media::guess_upload_type("audio.WAV"), "voice");

    // 视频
    assert_eq!(media::guess_upload_type("clip.mp4"), "video");
    assert_eq!(media::guess_upload_type("clip.MOV"), "video");

    // 其他
    assert_eq!(media::guess_upload_type("doc.pdf"), "file");
    assert_eq!(media::guess_upload_type("noext"), "file");
    assert_eq!(media::guess_upload_type(""), "file");

    // HTTP URL 带查询参数
    assert_eq!(media::guess_upload_type("https://cdn.example.com/photo.png?token=abc"), "image");
}

#[test]
fn guess_filename_local_and_url() {
    // 本地路径
    assert_eq!(media::guess_filename("/tmp/photo.jpg", "image"), "photo.jpg");
    assert_eq!(media::guess_filename("relative/doc.pdf", "file"), "doc.pdf");

    // HTTP URL
    assert_eq!(media::guess_filename("https://cdn.example.com/images/cat.png?v=1", "image"), "cat.png");

    // file:// 协议
    assert_eq!(media::guess_filename("file:///home/user/report.pdf", "file"), "report.pdf");

    // 无文件名 fallback
    assert_eq!(media::guess_filename("", "image"), "image.jpg");
    assert_eq!(media::guess_filename("", "voice"), "audio.amr");
    assert_eq!(media::guess_filename("", "video"), "video.mp4");
    assert_eq!(media::guess_filename("", "file"), "file.bin");
}

#[test]
fn guess_mime_type_known_and_unknown() {
    assert_eq!(media::guess_mime_type("photo.png"), "image/png");
    assert_eq!(media::guess_mime_type("photo.jpg"), "image/jpeg");
    assert_eq!(media::guess_mime_type("photo.JPEG"), "image/jpeg");
    assert_eq!(media::guess_mime_type("audio.mp3"), "audio/mpeg");
    assert_eq!(media::guess_mime_type("clip.mp4"), "video/mp4");
    assert_eq!(media::guess_mime_type("doc.pdf"), "application/octet-stream");
    assert_eq!(media::guess_mime_type("noext"), "application/octet-stream");
}
