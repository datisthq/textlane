use textlane_core::nlcst::Node;
use textlane_core::{Allocator, File, Lines, Position, UnistPosition};

/// A position as unist writes it: a byte span through the file's source, a unist position
/// as it is. Builds `Lines` each call: many messages go through `place_messages` instead.
pub(crate) fn to_place(file: &File, position: &Position<'_>) -> Option<UnistPosition> {
    match position {
        Position::Unist(position) => Some((*position).clone()),
        Position::Span { .. } => {
            let lines = Lines::new(file.value.as_deref()?);
            Some(lines.to_unist_position(position))
        }
    }
}

/// From the first node's start to the last one's end, as unist-util-position's
/// `pointStart` and `pointEnd` read them.
pub(crate) fn join_positions<'a>(
    first: &Node<'a>,
    last: &Node<'a>,
    alloc: &'a Allocator,
) -> Option<Position<'a>> {
    match (*first.position()?, *last.position()?) {
        (Position::Span { start, .. }, Position::Span { end, .. }) => {
            Some(Position::span(start, end))
        }
        (Position::Unist(start), Position::Unist(end)) => {
            Some(Position::Unist(alloc.alloc(UnistPosition {
                start: start.start,
                end: end.end,
            })))
        }
        // A span and a foreign position share no source to join them in
        _ => None,
    }
}
