use textlane_core::hast;
use textlane_core::mdast::{Node, Table};
use textlane_core::{ArenaVec, Position, UnistPosition};

use crate::utils::converter::state::Converter;

/// A table becomes `table`, its first row in `thead` and the rest in `tbody`.
pub(crate) fn convert_table<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Table<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let mut rows = children.into_iter();
    let mut content = ArenaVec::new_in(converter.alloc);
    if let Some(first) = rows.next() {
        let mut head = ArenaVec::new_in(converter.alloc);
        head.push(first);
        let position = node.children.first().and_then(Node::position).copied();
        let head = converter.wrap(head, true);
        content.push(converter.element("thead", head, position));
    }
    let mut body = ArenaVec::new_in(converter.alloc);
    body.extend(rows);
    if !body.is_empty() {
        let position = match (node.children.get(1), node.children.last()) {
            (Some(first), Some(last)) => join_positions(converter, first, last),
            _ => None,
        };
        let body = converter.wrap(body, true);
        content.push(converter.element("tbody", body, position));
    }
    let content = converter.wrap(content, true);
    converter.element("table", content, node.position)
}

// From the start of `first` to the end of `last`, when both have one
fn join_positions<'a>(
    converter: &Converter<'a, '_>,
    first: &Node<'a>,
    last: &Node<'a>,
) -> Option<Position<'a>> {
    match (first.position()?, last.position()?) {
        (Position::Span { start, .. }, Position::Span { end, .. }) => {
            Some(Position::span(*start, *end))
        }
        (Position::Unist(start), Position::Unist(end)) => {
            Some(Position::Unist(converter.alloc.alloc(UnistPosition {
                start: start.start,
                end: end.end,
            })))
        }
        _ => None,
    }
}
