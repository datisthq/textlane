use textlane_core::mdast::{Node, Root};
use textlane_core::{Allocator, ArenaString, ArenaVec, Point, Position};

use crate::models::event::{self, Event};
use crate::models::token::Name;

/// Builds mdast from events: a stack of open nodes, each attached to its parent when it
/// closes, so reaching the innermost one costs nothing however deep the tree is.
pub(crate) struct Compiler<'a, 'e> {
    pub(crate) events: &'e [Event],
    pub(crate) source: &'a str,
    pub(crate) alloc: &'a Allocator,
    pub(crate) index: usize,
    /// The marker of the character reference being compiled: `&`, `#` or `x`.
    pub(crate) character_reference_marker: u8,
    /// Whether a hard break just closed, so the line ending after it joins it.
    pub(crate) hard_break_after: bool,
    stack: Vec<Node<'a>>,
}

impl<'a, 'e> Compiler<'a, 'e> {
    pub(crate) fn new(
        events: &'e [Event],
        source: &'a str,
        alloc: &'a Allocator,
    ) -> Self {
        let (start, end) = match (events.first(), events.last()) {
            (Some(first), Some(last)) => (to_point(&first.point), to_point(&last.point)),
            _ => (Point::new(0, 0), Point::new(0, 0)),
        };
        let root = Node::Root(Root {
            children: ArenaVec::new_in(alloc),
            position: Some(Position::span(start, end)),
        });
        Self {
            events,
            source,
            alloc,
            index: 0,
            character_reference_marker: 0,
            hard_break_after: false,
            stack: vec![root],
        }
    }

    pub(crate) fn finish(mut self) -> Node<'a> {
        // Events always balance, but attach anything left open rather than lose it
        while self.stack.len() > 1 {
            self.attach_tail();
        }
        match self.stack.pop() {
            Some(root) => root,
            None => unreachable!("the root is never popped"),
        }
    }

    /// The innermost open node.
    pub(crate) fn tail_mut(&mut self) -> &mut Node<'a> {
        match self.stack.last_mut() {
            Some(node) => node,
            None => unreachable!("the root is never popped"),
        }
    }

    /// Open `child` in the innermost open node, starting at the current event.
    pub(crate) fn tail_push(&mut self, mut child: Node<'a>) {
        let point = to_point(&self.events[self.index].point);
        if child.position().is_none() {
            *child.position_mut() = Some(Position::span(point, point));
        }
        self.stack.push(child);
    }

    /// Open `child` like `tail_push`, but starting after the spaces or tabs that open the
    /// current token: micromark eats a flow construct's indent as a line prefix before it.
    pub(crate) fn tail_push_after_prefix(&mut self, mut child: Node<'a>) {
        let mut index = self.index + 1;
        while index + 1 < self.events.len()
            && self.events[index].name == Name::SPACE_OR_TAB
            && self.events[index].kind == event::Kind::Enter
        {
            index += 2;
        }
        let start = to_point(&self.events[index - 1].point);
        *child.position_mut() = Some(Position::span(start, start));
        self.tail_push(child);
    }

    /// Reopen the last child of the innermost open node.
    pub(crate) fn tail_push_again(&mut self) {
        if let Some(last) = self
            .tail_mut()
            .children_mut()
            .and_then(|children| children.pop())
        {
            self.stack.push(last);
        }
    }

    /// Close the innermost open node at the current event.
    pub(crate) fn tail_pop(&mut self) {
        let end = to_point(&self.events[self.index].point);
        if let Some(Position::Span { end: node_end, .. }) = self.tail_mut().position_mut()
        {
            *node_end = end;
        }
        self.attach_tail();
    }

    // Move the innermost open node into its parent's children
    fn attach_tail(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        if let Some(node) = self.stack.pop()
            && let Some(children) = self.tail_mut().children_mut()
        {
            children.push(node);
        }
    }

    /// Set the end of the innermost open node's last child to the current event.
    pub(crate) fn extend_last_child(&mut self) {
        let end = to_point(&self.events[self.index].point);
        if let Some(last) = self
            .tail_mut()
            .children_mut()
            .and_then(|children| children.last_mut())
            && let Some(Position::Span { end: last_end, .. }) = last.position_mut()
        {
            *last_end = end;
        }
    }

    /// The source text of the token whose exit is the current event.
    pub(crate) fn slice(&self) -> &'a str {
        let end = &self.events[self.index].point;
        let name = self.events[self.index].name;
        let mut start = self.index;
        while start > 0
            && !(self.events[start].kind == event::Kind::Enter
                && self.events[start].name == name)
        {
            start -= 1;
        }
        self.source
            .get(self.events[start].point.index..end.index)
            .unwrap_or_default()
    }

    /// `value` followed by `addition`, borrowing the source when they are adjacent in it.
    pub(crate) fn join(&self, value: &'a str, addition: &'a str) -> &'a str {
        if value.is_empty() {
            return addition;
        }
        if let (Some(start), Some(next)) =
            (self.offset_of(value), self.offset_of(addition))
            && start + value.len() == next
            && let Some(joined) = self.source.get(start..next + addition.len())
        {
            return joined;
        }
        let mut joined =
            ArenaString::with_capacity_in(value.len() + addition.len(), self.alloc);
        joined.push_str(value);
        joined.push_str(addition);
        joined.into_bump_str()
    }

    fn offset_of(&self, value: &str) -> Option<usize> {
        let base = self.source.as_ptr() as usize;
        let at = value.as_ptr() as usize;
        (at >= base && at + value.len() <= base + self.source.len()).then(|| at - base)
    }
}

pub(crate) fn to_point(point: &event::Point) -> Point {
    Point::new(point.index, point.vs)
}
