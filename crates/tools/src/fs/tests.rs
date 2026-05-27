//! 文件系统工具测试

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
    let result = tool.execute(&test_ctx(), json!({"path": file_path.to_str().unwrap()})).await.unwrap();

    assert!(result.contains("1: hello world"), "should contain line-numbered content: {result}");
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
async fn read_file_with_offset_limit() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("lines.txt");
    std::fs::write(&file_path, "line1\nline2\nline3\nline4\nline5\n").unwrap();

    let tool = ReadFileTool::new(dir.path(), None::<PathBuf>);

    // Read lines 2-3
    let result =
        tool.execute(&test_ctx(), json!({"path": file_path.to_str().unwrap(), "offset": 2, "limit": 2})).await.unwrap();

    assert!(result.contains("2: line2"), "should contain line 2: {result}");
    assert!(result.contains("3: line3"), "should contain line 3: {result}");
    assert!(!result.contains("1: line1"), "should not contain line 1: {result}");
    assert!(result.contains("Use offset=4"), "should suggest next offset: {result}");
}

#[tokio::test]
async fn read_file_offset_beyond_end() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("short.txt");
    std::fs::write(&file_path, "one\ntwo\n").unwrap();

    let tool = ReadFileTool::new(dir.path(), None::<PathBuf>);
    let result = tool.execute(&test_ctx(), json!({"path": file_path.to_str().unwrap(), "offset": 100})).await.unwrap();

    assert!(result.contains("beyond the end"), "should indicate offset beyond end: {result}");
}
