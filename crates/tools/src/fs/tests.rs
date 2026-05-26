//! ReadFileTool 大小限制测试

use serde_json::json;

use super::*;

fn test_ctx() -> ToolContext {
    ToolContext::new("test", "test")
}

#[tokio::test]
async fn read_normal_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("small.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    let tool = ReadFileTool::new(dir.path(), None::<PathBuf>);
    let result = tool.execute(&test_ctx(), json!({"path": file_path.to_str().unwrap()})).await;

    assert_eq!(result.unwrap(), "hello world");
}

#[tokio::test]
async fn read_oversized_file_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("huge.bin");
    // 创建 > 512KB 的文件
    let data = vec![b'x'; MAX_BYTES + 1];
    std::fs::write(&file_path, &data).unwrap();

    let tool = ReadFileTool::new(dir.path(), None::<PathBuf>);
    let result = tool.execute(&test_ctx(), json!({"path": file_path.to_str().unwrap()})).await;

    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("文件过大"), "expected '文件过大' in error: {msg}");
}

#[tokio::test]
async fn read_large_file_truncated() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("large.txt");
    // 创建 130K 字符的 ASCII 文件（130KB < 512KB，不会被预读拒绝）
    let char_count = MAX_CHARS + 2000;
    let data = "a".repeat(char_count);
    std::fs::write(&file_path, &data).unwrap();

    let tool = ReadFileTool::new(dir.path(), None::<PathBuf>);
    let result = tool.execute(&test_ctx(), json!({"path": file_path.to_str().unwrap()})).await;

    let content = result.unwrap();
    assert!(content.contains("[文件已截断"), "expected truncation notice in output");
    // 截断后的内容不应包含完整原文
    assert!(content.len() < data.len(), "truncated content should be shorter than original");
}
