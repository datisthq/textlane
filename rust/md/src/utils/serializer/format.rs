use textlane_core::mdast::{Code, Heading, Link, Node};

use crate::helpers::decode::decode_string;
use crate::helpers::html::{BLOCK_NAMES, RAW_NAMES};
use crate::utils::serializer::settings::Settings;

/// Whether `code` can be written indented: fences are off and nothing about it needs them.
pub(crate) fn format_code_as_indented(code: &Code, settings: &Settings) -> bool {
    let value = code.value;
    if settings.fences || value.is_empty() || code.lang.is_some() {
        return false;
    }
    if !value
        .bytes()
        .any(|byte| !matches!(byte, b'\n' | b'\r' | b' '))
    {
        return false;
    }
    let lines: Vec<&str> = split_lines(value).collect();
    let is_blank = |line: &&str| line.bytes().all(|byte| matches!(byte, b'\t' | b' '));
    !(lines.first().is_some_and(is_blank) || lines.last().is_some_and(is_blank))
}

/// Whether `heading` is written as a setext heading.
pub(crate) fn format_heading_as_setext(heading: &Heading, settings: &Settings) -> bool {
    if heading.depth >= 3 {
        return false;
    }
    let mut has_text = false;
    let mut has_break = false;
    let mut stack: Vec<&Node> = heading.children.iter().rev().collect();
    while let Some(node) = stack.pop() {
        if let Some(value) = read_literal(node) {
            has_text = has_text || !value.is_empty();
            has_break = has_break || value.contains(['\n', '\r']);
        }
        if let Node::Image(image) = node {
            has_text = has_text || image.alt.is_some_and(|alt| !alt.is_empty());
        }
        if let Node::ImageReference(image) = node {
            has_text = has_text || image.alt.is_some_and(|alt| !alt.is_empty());
        }
        has_break = has_break || matches!(node, Node::Break(_));
        if let Some(children) = node.children() {
            stack.extend(children.iter().rev());
        }
    }
    has_text && (settings.setext || has_break)
}

/// Whether `link` is written as an autolink, `<https://example.com>`.
pub(crate) fn format_link_as_autolink(link: &Link, settings: &Settings) -> bool {
    let [Node::Text(text)] = &link.children[..] else {
        return false;
    };
    let raw = text.value;
    let url = link.url;
    let scheme_end = url.find(':');
    !settings.resource_link
        && !url.is_empty()
        && link.title.is_none()
        && (raw == url || url.strip_prefix("mailto:") == Some(raw))
        && scheme_end.is_some_and(|end| {
            end >= 2
                && url.as_bytes()[0].is_ascii_alphabetic()
                && url.as_bytes()[1..end].iter().all(|byte| {
                    byte.is_ascii_alphabetic() || matches!(byte, b'+' | b'-' | b'.')
                })
        })
        && !url.chars().any(|character| {
            character <= ' ' || matches!(character, '<' | '>' | '\u{7F}')
        })
}

/// The kind of HTML block, 1 to 7, `value` would start, as micromark numbers them.
pub(crate) fn read_html_kind(value: &str) -> Option<u8> {
    let bytes = value.as_bytes();
    if bytes.first() != Some(&b'<') {
        return None;
    }
    match bytes.get(1) {
        Some(b'!') => {
            return match bytes.get(2) {
                Some(b'-') => Some(2),
                Some(b'[') => Some(5),
                Some(byte) if byte.is_ascii_alphabetic() => Some(4),
                _ => None,
            };
        }
        Some(b'?') => return Some(3),
        _ => {}
    }
    let closing = bytes.get(1) == Some(&b'/');
    let start = if closing { 2 } else { 1 };
    if !bytes.get(start).is_some_and(u8::is_ascii_alphabetic) {
        return None;
    }
    let mut end = start;
    while bytes
        .get(end)
        .is_some_and(|byte| *byte == b'-' || byte.is_ascii_alphanumeric())
    {
        end += 1;
    }
    let name = value[start..end].to_ascii_lowercase();
    if !closing && bytes.get(end) != Some(&b'/') && RAW_NAMES.contains(&name.as_str()) {
        return Some(1);
    }
    Some(if BLOCK_NAMES.contains(&name.as_str()) {
        6
    } else {
        7
    })
}

/// Whether `node` is phrasing content, as mdast-util-phrasing decides.
pub(crate) fn is_phrasing(node: &Node) -> bool {
    matches!(
        node,
        Node::Break(_)
            | Node::Emphasis(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::InlineCode(_)
            | Node::Link(_)
            | Node::LinkReference(_)
            | Node::Strong(_)
            | Node::Text(_)
    )
}

/// The label a reference or definition is written with: its `label`, or its decoded
/// `identifier`.
pub(crate) fn read_association(identifier: &str, label: Option<&str>) -> String {
    match label {
        Some(label) if !label.is_empty() => label.to_string(),
        _ if identifier.is_empty() => String::new(),
        _ => decode_string(identifier),
    }
}

/// The longest run of `character` in `value`.
pub(crate) fn measure_longest_streak(value: &str, character: char) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for each in value.chars() {
        if each == character {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

/// Split on `\r\n`, `\n` and `\r`.
pub(crate) fn split_lines(value: &str) -> impl Iterator<Item = &str> {
    let mut rest = Some(value);
    std::iter::from_fn(move || {
        let current = rest?;
        match current.find(['\n', '\r']) {
            Some(index) => {
                let width = if current[index..].starts_with("\r\n") {
                    2
                } else {
                    1
                };
                rest = Some(&current[index + width..]);
                Some(&current[..index])
            }
            None => {
                rest = None;
                Some(current)
            }
        }
    })
}

fn read_literal<'a>(node: &Node<'a>) -> Option<&'a str> {
    match node {
        Node::Text(node) => Some(node.value),
        Node::InlineCode(node) => Some(node.value),
        Node::Code(node) => Some(node.value),
        Node::Html(node) => Some(node.value),
        _ => None,
    }
}
