//! 飞书交互式卡片消息文本提取。
//!
//! 飞书 interactive 消息的 `elements` 是二维数组（行列表，每行是元素列表），
//! 每个元素通过 `tag` 字段标识类型（markdown、div、button 等）。

use serde::Deserialize;
use tracing::warn;

// ── 数据模型 ──────────────────────────────────────────────────────────

/// 交互式卡片消息顶层结构。
#[derive(Deserialize, Default)]
#[serde(default)]
struct InteractiveContent {
    title: Option<TitleValue>,
    elements: Option<Vec<ElementRow>>,
    card: Option<Box<InteractiveContent>>,
    header: Option<Header>,
}

/// title 字段：可以是纯字符串或 `{content, text}` 对象。
#[derive(Deserialize)]
#[serde(untagged)]
enum TitleValue {
    Text(String),
    Rich(RichText),
}

/// 通用富文本对象，优先取 `content`，其次 `text`。
#[derive(Deserialize, Default)]
#[serde(default)]
struct RichText {
    content: Option<String>,
    text: Option<String>,
}

impl RichText {
    fn effective_text(&self) -> Option<&str> {
        self.content.as_deref().filter(|s| !s.is_empty()).or_else(|| self.text.as_deref().filter(|s| !s.is_empty()))
    }
}

/// elements 行：可以是元素数组（二维数组的一行）或单个元素（一维数组兼容）。
#[derive(Deserialize)]
#[serde(untagged)]
enum ElementRow {
    Array(Vec<CardElement>),
    Single(Box<CardElement>),
}

/// 卡片头部。
#[derive(Deserialize, Default)]
#[serde(default)]
struct Header {
    title: Option<RichText>,
}

/// 单个卡片元素，按 `tag` 字段区分类型。
#[derive(Deserialize)]
#[serde(tag = "tag", rename_all = "snake_case")]
enum CardElement {
    /// markdown 文本
    Markdown { content: Option<String> },
    /// lark_md 文本（与 markdown 结构相同）
    LarkMd { content: Option<String> },
    /// 纯文本
    PlainText { content: Option<String> },
    /// 文本块（含 fields）
    Div { text: Option<TextValue>, fields: Option<Vec<Field>> },
    /// 链接
    A { href: Option<String>, text: Option<String> },
    /// 按钮
    Button { text: Option<RichText>, url: Option<String>, multi_url: Option<MultiUrl> },
    /// 图片
    Img { alt: Option<RichText> },
    /// 备注（含子元素）
    Note { elements: Option<Vec<CardElement>> },
    /// 多列布局
    ColumnSet { columns: Option<Vec<Column>> },
    /// 未知 tag
    #[serde(other)]
    Unknown,
}

/// text 字段：可以是纯字符串或富文本对象。
#[derive(Deserialize)]
#[serde(untagged)]
enum TextValue {
    Plain(String),
    Rich(RichText),
}

/// 多平台链接。
#[derive(Deserialize, Default)]
#[serde(default)]
struct MultiUrl {
    url: Option<String>,
}

/// div 的 field 项。
#[derive(Deserialize, Default)]
#[serde(default)]
struct Field {
    text: Option<RichText>,
}

/// column_set 的列。
#[derive(Deserialize, Default)]
#[serde(default)]
struct Column {
    elements: Option<Vec<CardElement>>,
}

// ── 提取逻辑 ──────────────────────────────────────────────────────────

/// 从 interactive 卡片消息 JSON 中提取文本内容。
///
/// 处理以下结构：
/// - `title`（字符串或 `{content/text}` 对象）
/// - `elements`（二维数组，递归提取每个元素）
/// - `card`（递归提取）
/// - `header.title`（`{content/text}` 对象）
pub fn extract_interactive_content(content: &serde_json::Value) -> String {
    let card: InteractiveContent = match serde_json::from_value(content.clone()) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to deserialize interactive content: {e}");
            return String::new();
        }
    };

    let mut parts = Vec::new();
    extract_parts(&card, &mut parts);
    parts.join("\n")
}

fn extract_parts(card: &InteractiveContent, parts: &mut Vec<String>) {
    // title
    if let Some(ref title) = card.title {
        let text = match title {
            TitleValue::Text(s) => Some(s.as_str()).filter(|s| !s.is_empty()),
            TitleValue::Rich(r) => r.effective_text(),
        };
        if let Some(t) = text {
            parts.push(format!("title: {t}"));
        }
    }

    // elements (二维数组，兼容一维)
    if let Some(ref rows) = card.elements {
        for row in rows {
            match row {
                ElementRow::Array(elements) => {
                    for element in elements {
                        extract_element(element, parts);
                    }
                }
                ElementRow::Single(element) => {
                    extract_element(element, parts);
                }
            }
        }
    }

    // card (递归)
    if let Some(ref inner) = card.card {
        extract_parts(inner, parts);
    }

    // header.title
    if let Some(ref header) = card.header
        && let Some(t) = header.title.as_ref().and_then(|t| t.effective_text())
    {
        parts.push(format!("title: {t}"));
    }
}

/// 按 variant 提取单个卡片元素的文本内容。
fn extract_element(el: &CardElement, parts: &mut Vec<String>) {
    match el {
        CardElement::Markdown { content } | CardElement::LarkMd { content } | CardElement::PlainText { content } => {
            if let Some(c) = content.as_deref()
                && !c.is_empty()
            {
                parts.push(c.to_string());
            }
        }
        CardElement::Div { text, fields } => {
            if let Some(text) = text {
                let t = match text {
                    TextValue::Plain(s) => Some(s.as_str()).filter(|s| !s.is_empty()),
                    TextValue::Rich(r) => r.effective_text(),
                };
                if let Some(t) = t {
                    parts.push(t.to_string());
                }
            }
            if let Some(fields) = fields {
                for field in fields {
                    if let Some(t) = field.text.as_ref().and_then(|t| t.effective_text()) {
                        parts.push(t.to_string());
                    }
                }
            }
        }
        CardElement::A { href, text } => {
            if let Some(href) = href
                && !href.is_empty()
            {
                parts.push(format!("link: {href}"));
            }
            if let Some(text) = text
                && !text.is_empty()
            {
                parts.push(text.clone());
            }
        }
        CardElement::Button { text, url, multi_url } => {
            if let Some(t) = text.as_ref().and_then(|r| r.effective_text()) {
                parts.push(t.to_string());
            }
            let link = url
                .as_deref()
                .filter(|s| !s.is_empty())
                .or_else(|| multi_url.as_ref().and_then(|m| m.url.as_deref()).filter(|s| !s.is_empty()));
            if let Some(u) = link {
                parts.push(format!("link: {u}"));
            }
        }
        CardElement::Img { alt } => {
            let text = alt.as_ref().and_then(|r| r.effective_text()).unwrap_or("[image]");
            parts.push(text.to_string());
        }
        CardElement::Note { elements } => {
            if let Some(elements) = elements {
                for ne in elements {
                    extract_element(ne, parts);
                }
            }
        }
        CardElement::ColumnSet { columns } => {
            if let Some(columns) = columns {
                for col in columns {
                    if let Some(ref elements) = col.elements {
                        for ce in elements {
                            extract_element(ce, parts);
                        }
                    }
                }
            }
        }
        CardElement::Unknown => {
            warn!("skipping unknown interactive card element");
        }
    }
}

#[cfg(test)]
mod tests;
