use textlane_core::mdast::{Node, Paragraph, Text};
use textlane_core::{Allocator, ArenaVec, Position};

use crate::models::options::DirectiveKind;
use crate::plugins::component::actions::directive::convert::DirectiveParts;
use crate::plugins::component::actions::text::read::read_text;

/// A text or leaf directive no rule converted, as the text it was written as: a text node,
/// or a paragraph of one; a container directive stays.
pub(crate) fn restore_directive<'a>(
    parts: DirectiveParts<'a>,
    source: Option<&str>,
    alloc: &'a Allocator,
) -> Node<'a> {
    if parts.kind == DirectiveKind::Container {
        return parts.into_node();
    }
    let value = match (parts.position, source) {
        (Some(Position::Span { start, end }), Some(source)) => {
            source.get(start.offset()..end.offset()).map(str::to_string)
        }
        _ => None,
    };
    // Without the source, the name and label are what remains of how it was written
    let value = value.unwrap_or_else(|| {
        let marker = if parts.kind == DirectiveKind::Leaf {
            "::"
        } else {
            ":"
        };
        let label: String = parts.children.iter().map(read_text).collect();
        match label.is_empty() {
            true => format!("{marker}{}", parts.name),
            false => format!("{marker}{}[{label}]", parts.name),
        }
    });
    let text = Node::Text(Text {
        value: alloc.alloc_str(&value),
        position: parts.position,
        data: None,
    });
    if parts.kind == DirectiveKind::Text {
        return text;
    }
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(text);
    Node::Paragraph(Paragraph {
        children,
        position: parts.position,
        data: None,
    })
}
