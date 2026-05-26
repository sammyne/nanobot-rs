//! 飞书 post（富文本）消息内容提取。
//!
//! 飞书 post 消息的 content 字段有三种格式：
//! 1. `{"post": {"zh_cn": {"title": "...", "content": [[...]]}}}` — 带 post 包裹
//! 2. `{"zh_cn": {"title": "...", "content": [[...]]}}` — 直接本地化
//! 3. `{"title": "...", "content": [[...]]}` — 直接内容
//!
//! 每行（content 的每个子数组）包含按 `tag` 区分的元素（text/a/at/img）。

use std::collections::HashMap;

use serde::Deserialize;
use tracing::warn;

// ── 数据模型 ──────────────────────────────────────────────────────────

/// Post 消息顶层结构。
///
/// 兼容三种格式：
/// 1. `{"post": {"zh_cn": {...}}}` — 带 post 包裹
/// 2. `{"zh_cn": {...}}` — 直接本地化
/// 3. `{"title": "...", "content": [...]}` — 直接内容
#[derive(Deserialize, Default)]
#[serde(default)]
struct PostContent {
    /// 格式 1 的外层包裹
    post: Option<HashMap<String, LocaleContent>>,
    /// 格式 2 的直接 locale keys
    zh_cn: Option<LocaleContent>,
    en_us: Option<LocaleContent>,
    ja_jp: Option<LocaleContent>,
    /// 格式 3 的直接内容字段
    title: Option<String>,
    content: Option<Vec<Vec<PostElement>>>,
}

/// 单个 locale 下的富文本内容。
#[derive(Deserialize, Default, Clone)]
#[serde(default)]
struct LocaleContent {
    /// 可选标题
    title: Option<String>,
    /// 二维数组：每行是一个元素列表
    content: Vec<Vec<PostElement>>,
}

/// Post 富文本元素，按 `tag` 字段区分类型。
#[derive(Deserialize, Clone)]
#[serde(tag = "tag", rename_all = "snake_case")]
enum PostElement {
    /// 纯文本
    Text {
        #[serde(default)]
        text: Option<String>,
    },
    /// 超链接
    A {
        #[serde(default)]
        text: Option<String>,
        #[allow(dead_code)]
        #[serde(default)]
        href: Option<String>,
    },
    /// @提及
    At {
        #[serde(default)]
        user_id: Option<String>,
    },
    /// 内嵌图片
    Img {
        #[serde(default)]
        image_key: Option<String>,
    },
    /// 未知 tag（兼容未来新增类型）
    #[serde(other)]
    Unknown,
}

// ── 提取逻辑 ──────────────────────────────────────────────────────────

/// 已知的 locale key，按 fallback 优先级排列。
const LOCALE_KEYS: &[&str] = &["zh_cn", "en_us", "ja_jp"];

/// 从 post 消息 JSON 中提取文本内容和图片 key 列表。
///
/// 返回 `(text, image_keys)`。兼容三种 post 格式。
pub fn extract_post_content(value: &serde_json::Value) -> (String, Vec<String>) {
    let pc: PostContent = match serde_json::from_value(value.clone()) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to deserialize post content: {e}");
            return (String::new(), Vec::new());
        }
    };

    let locale = resolve_locale(&pc);

    let Some(locale) = locale else {
        return (String::new(), Vec::new());
    };

    extract_from_locale(&locale)
}

/// 按优先级查找有效的 [`LocaleContent`]。
fn resolve_locale(pc: &PostContent) -> Option<LocaleContent> {
    // 格式 1：从 post HashMap 中按 locale fallback 取值
    if let Some(ref map) = pc.post {
        for key in LOCALE_KEYS {
            if let Some(lc) = map.get(*key)
                && !lc.content.is_empty()
            {
                return Some(lc.clone());
            }
        }
        // fallback：HashMap 中任意首个值
        if let Some(lc) = map.values().find(|lc| !lc.content.is_empty()) {
            return Some(lc.clone());
        }
    }

    // 格式 2：直接 locale 字段
    for lc in [&pc.zh_cn, &pc.en_us, &pc.ja_jp].into_iter().flatten() {
        if !lc.content.is_empty() {
            return Some(lc.clone());
        }
    }

    // 格式 3：直接 title + content 字段
    if let Some(ref rows) = pc.content
        && !rows.is_empty()
    {
        return Some(LocaleContent { title: pc.title.clone(), content: rows.clone() });
    }

    None
}

/// 从 [`LocaleContent`] 中提取文本和图片 key。
fn extract_from_locale(locale: &LocaleContent) -> (String, Vec<String>) {
    let mut text_parts = Vec::new();
    let mut image_keys = Vec::new();

    // title 拼接到文本开头
    if let Some(ref title) = locale.title
        && !title.is_empty()
    {
        text_parts.push(title.clone());
    }

    for row in &locale.content {
        for element in row {
            match element {
                PostElement::Text { text } => {
                    if let Some(t) = text.as_deref().filter(|s| !s.is_empty()) {
                        text_parts.push(t.to_string());
                    }
                }
                PostElement::A { text, .. } => {
                    if let Some(t) = text.as_deref().filter(|s| !s.is_empty()) {
                        text_parts.push(t.to_string());
                    }
                }
                PostElement::At { user_id } => {
                    let mention = user_id.as_deref().unwrap_or("user");
                    text_parts.push(format!("@{mention}"));
                }
                PostElement::Img { image_key } => {
                    if let Some(key) = image_key.as_deref().filter(|s| !s.is_empty()) {
                        image_keys.push(key.to_string());
                    }
                }
                PostElement::Unknown => {}
            }
        }
    }

    (text_parts.join(" "), image_keys)
}

#[cfg(test)]
mod tests;
