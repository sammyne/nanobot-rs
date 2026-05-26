use serde_json::json;

use super::*;

/// 格式 1：`{"post": {"zh_cn": {...}}}` 包裹格式
#[test]
fn format1_post_wrapper() {
    let value = json!({
        "post": {
            "zh_cn": {
                "title": "日报",
                "content": [
                    [
                        {"tag": "text", "text": "完成"},
                        {"tag": "img", "image_key": "img_1"}
                    ]
                ]
            }
        }
    });

    let (text, image_keys) = extract_post_content(&value);
    assert_eq!(text, "日报 完成");
    assert_eq!(image_keys, vec!["img_1"]);
}

/// 格式 2：`{"zh_cn": {...}}` 直接本地化格式
#[test]
fn format2_direct_locale() {
    let value = json!({
        "zh_cn": {
            "title": "周报",
            "content": [
                [
                    {"tag": "text", "text": "本周进展"},
                    {"tag": "a", "text": "详情", "href": "https://example.com"}
                ]
            ]
        }
    });

    let (text, image_keys) = extract_post_content(&value);
    assert_eq!(text, "周报 本周进展 详情");
    assert!(image_keys.is_empty());
}

/// 格式 3：`{"title": "...", "content": [...]}` 直接内容格式
#[test]
fn format3_direct_content() {
    let value = json!({
        "title": "Daily",
        "content": [
            [
                {"tag": "text", "text": "report"},
                {"tag": "img", "image_key": "img_a"},
                {"tag": "img", "image_key": "img_b"}
            ]
        ]
    });

    let (text, image_keys) = extract_post_content(&value);
    assert_eq!(text, "Daily report");
    assert_eq!(image_keys, vec!["img_a", "img_b"]);
}

/// 空内容返回空字符串和空 vec
#[test]
fn empty_content() {
    let value = json!({});
    let (text, image_keys) = extract_post_content(&value);
    assert!(text.is_empty());
    assert!(image_keys.is_empty());
}

/// 无 title 时仅返回元素文本
#[test]
fn no_title() {
    let value = json!({
        "zh_cn": {
            "content": [
                [{"tag": "text", "text": "hello"}]
            ]
        }
    });

    let (text, image_keys) = extract_post_content(&value);
    assert_eq!(text, "hello");
    assert!(image_keys.is_empty());
}

/// @提及元素提取为 @user_id
#[test]
fn at_mention() {
    let value = json!({
        "content": [
            [
                {"tag": "text", "text": "请"},
                {"tag": "at", "user_id": "ou_123"},
                {"tag": "text", "text": "查看"}
            ]
        ]
    });

    let (text, _) = extract_post_content(&value);
    assert_eq!(text, "请 @ou_123 查看");
}

/// 多行内容拼接
#[test]
fn multi_row() {
    let value = json!({
        "post": {
            "zh_cn": {
                "content": [
                    [{"tag": "text", "text": "第一行"}],
                    [{"tag": "text", "text": "第二行"}]
                ]
            }
        }
    });

    let (text, _) = extract_post_content(&value);
    assert_eq!(text, "第一行 第二行");
}

/// 未知 tag 被安全忽略
#[test]
fn unknown_tag_ignored() {
    let value = json!({
        "content": [
            [
                {"tag": "text", "text": "before"},
                {"tag": "emotion", "emoji_type": "SMILE"},
                {"tag": "text", "text": "after"}
            ]
        ]
    });

    let (text, _) = extract_post_content(&value);
    assert_eq!(text, "before after");
}

/// 格式 1 locale fallback：zh_cn 不存在时使用 en_us
#[test]
fn locale_fallback_to_en_us() {
    let value = json!({
        "post": {
            "en_us": {
                "title": "Report",
                "content": [[{"tag": "text", "text": "done"}]]
            }
        }
    });

    let (text, _) = extract_post_content(&value);
    assert_eq!(text, "Report done");
}
