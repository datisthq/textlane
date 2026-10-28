use textlane_core::mdast::{Node, Root};
use textlane_core::{Allocator, ArenaString, ArenaVec, Point, Position};

use crate::models::event::{self, Event};
use crate::models::token::Name;

/// Builds mdast from events: a stack of open nodes in the tree being built.
pub(crate) struct Compiler<'a, 'e> {
    pub(crate) events: &'e [Event],
    pub(crate) source: &'a str,
    pub(crate) alloc: &'a Allocator,
    pub(crate) index: usize,
    tree: Node<'a>,
    stack: Vec<usize>,
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
        let tree = Node::Root(Root {
            children: ArenaVec::new_in(alloc),
            position: Some(Position::span(start, end)),
        });
        Self {
            events,
            source,
            alloc,
            index: 0,
            tree,
            stack: vec![],
        }
    }

    pub(crate) fn finish(self) -> Node<'a> {
        self.tree
    }

    /// The innermost open node.
    pub(crate) fn tail_mut(&mut self) -> &mut Node<'a> {
        delve_mut(&mut self.tree, &self.stack)
    }

    /// Open `child` in the innermost open node, starting at the current event.
    pub(crate) fn tail_push(&mut self, mut child: Node<'a>) {
        let point = to_point(&self.events[self.index].point);
        if child.position().is_none() {
            *child.position_mut() = Some(Position::span(point, point));
        }
        let parent = delve_mut(&mut self.tree, &self.stack);
        if let Some(children) = parent.children_mut() {
            children.push(child);
            self.stack.push(children.len() - 1);
        }
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
        let parent = delve_mut(&mut self.tree, &self.stack);
        if let Some(children) = parent.children() {
            let last = children.len().saturating_sub(1);
            self.stack.push(last);
        }
    }

    /// Close the innermost open node at the current event.
    pub(crate) fn tail_pop(&mut self) {
        let end = to_point(&self.events[self.index].point);
        let node = delve_mut(&mut self.tree, &self.stack);
        if let Some(Position::Span { end: node_end, .. }) = node.position_mut() {
            *node_end = end;
        }
        self.stack.pop();
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

fn delve_mut<'t, 'a>(node: &'t mut Node<'a>, stack: &[usize]) -> &'t mut Node<'a> {
    let Some((first, rest)) = stack.split_first() else {
        return node;
    };
    if !node
        .children()
        .is_some_and(|children| *first < children.len())
    {
        return node;
    }
    match node
        .children_mut()
        .and_then(|children| children.get_mut(*first))
    {
        Some(child) => delve_mut(child, rest),
        None => unreachable!("the child was just checked to exist"),
    }
}

pub(crate) fn to_point(point: &event::Point) -> Point {
    Point::new(point.index, point.vs)
}
