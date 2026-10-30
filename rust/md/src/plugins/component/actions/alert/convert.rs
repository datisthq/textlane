use textlane_core::mdast::{Blockquote, Node};
use textlane_core::{Allocator, ArenaVec};

use crate::models::options::AlertRule;
use crate::plugins::component::actions::element::create::create_element;

/// The component a GitHub alert, a block quote opening with `[!TYPE]`, becomes, its marker
/// dropped; any other block quote comes back as it was.
pub(crate) fn convert_alert<'a>(
    rule: &AlertRule,
    mut quote: Blockquote<'a>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let Some((value, marker)) = read_marker(rule, &quote) else {
        return Node::Blockquote(quote);
    };
    if let Some(Node::Paragraph(paragraph)) = quote.children.first_mut() {
        if let Some(Node::Text(text)) = paragraph.children.first_mut() {
            text.value = &text.value[marker..];
            // the text no longer starts where it was written
            text.position = None;
            if text.value.is_empty() {
                paragraph.children.remove(0);
            }
        }
        if paragraph.children.is_empty() {
            quote.children.remove(0);
        }
    }
    let attributes = vec![(rule.type_as.clone(), value)];
    let children = std::mem::replace(&mut quote.children, ArenaVec::new_in(alloc));
    let name = alloc.alloc_str(&rule.component);
    create_element(name, attributes, children, quote.position, false, alloc)
}

// `[!TYPE]` and the white space after it, as livemark's `^\[!(\w+)\]\s*` reads it: the
// type's value and how many bytes the marker spans
fn read_marker(rule: &AlertRule, quote: &Blockquote<'_>) -> Option<(String, usize)> {
    let Some(Node::Paragraph(paragraph)) = quote.children.first() else {
        return None;
    };
    let Some(Node::Text(text)) = paragraph.children.first() else {
        return None;
    };
    let rest = text.value.strip_prefix("[!")?;
    let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    let kind = &rest[..end];
    let after = rest[end..].strip_prefix(']')?;
    if kind.is_empty() {
        return None;
    }
    let value = rule.types.get(kind)?.clone();
    let spaces = after.len() - after.trim_start().len();
    Some((value, text.value.len() - after.len() + spaces))
}
