use textlane_core::mdast::{Node, Paragraph, ReferenceType, Root};
use textlane_core::{Allocator, ArenaString, ArenaVec, Point, Position};

use std::collections::HashSet;

use crate::helpers::slice::Slice;
use crate::helpers::spread::{find_loose, find_trailing_line_endings};
use crate::models::event::{self, Event};
use crate::models::token::Name;

/// What a link or image is, worked out as its label and what follows it compile.
#[derive(Debug, Default)]
pub(crate) struct Media<'a> {
    /// A label closed without a resource after it: it is a reference.
    pub(crate) in_reference: bool,
    pub(crate) reference_type: Option<ReferenceType>,
    pub(crate) identifier: &'a str,
    pub(crate) label: Option<&'a str>,
    /// The exit of the label's text, read only if this turns out to be a reference:
    /// reading every label would cost each nested image all those inside it.
    pub(crate) label_text: Option<usize>,
}

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
    /// Whether a setext heading's text just closed, so the line ending before its
    /// underline is not text.
    pub(crate) heading_setext_text_after: bool,
    // The value of the open or last text node while pieces that are not adjacent in the
    // source join it: copying the value on each would make many escapes quadratic
    text_builder: Option<ArenaString<'a>>,
    // The enter of each exit, so slicing a token never walks back over the ones inside
    enters: Vec<usize>,
    // Where container prefixes start and end on prefixed lines, in order
    prefixes: Vec<(usize, usize)>,
    /// What each link or image being compiled turns out to be.
    pub(crate) media: Vec<Media<'a>>,
    /// Whether a code fence's opening fence closed, so the code after it is buffered.
    pub(crate) code_fence_seen: bool,
    /// Whether a GFM table is open, where code spans read `\|` as `|`.
    pub(crate) in_table: bool,
    /// The enters of loose lists and list items.
    pub(crate) loose: HashSet<usize>,
    /// Whether a GFM task list check is in the open paragraph, whose first text then
    /// loses the space after it.
    pub(crate) task_list_check_seen: bool,
    // The first line ending of the trailing lines before each event, without and with
    // block quote prefixes
    trailing_line_endings: [Vec<Option<usize>>; 2],
    stack: Vec<Node<'a>>,
}

impl<'a, 'e> Compiler<'a, 'e> {
    pub(crate) fn new(
        events: &'e [Event],
        source: &'a str,
        alloc: &'a Allocator,
        prefixes: Vec<(usize, usize)>,
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
            code_fence_seen: false,
            in_table: false,
            media: vec![],
            enters: find_enters(events),
            prefixes,
            text_builder: None,
            heading_setext_text_after: false,
            loose: find_loose(events),
            task_list_check_seen: false,
            trailing_line_endings: [
                find_trailing_line_endings(events, false),
                find_trailing_line_endings(events, true),
            ],
            stack: vec![root],
        }
    }

    pub(crate) fn finish(mut self) -> Node<'a> {
        self.flush_text();
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
        self.flush_text();
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
        // A list starts where its first item does, and that item after its indent
        // A setext heading after definitions starts where they do, lines above its text
        let line = self.events[self.index].point.line;
        while index < self.events.len()
            && self.events[index].kind == event::Kind::Enter
            && (self.events[index].name == Name::LIST_ITEM
                || (self.events[index].name == Name::HEADING_SETEXT_TEXT
                    && self.events[index].point.line == line))
        {
            index += 1;
        }
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
        // A text node being built stays open to more pieces as its parent's last child
        if !matches!(self.stack.last(), Some(Node::Text(_))) {
            self.flush_text();
        }
        let end = to_point(&self.events[self.index].point);
        if let Some(Position::Span { end: node_end, .. }) = self.tail_mut().position_mut()
        {
            *node_end = end;
        }
        self.attach_tail();
    }

    /// Close the innermost open node at the first line ending of the blank lines before
    /// the current event, as micromark ends lists and mdast-util-from-markdown list items;
    /// `prefixes` also passes block quote prefixes, as items do.
    pub(crate) fn tail_pop_before_trailing_lines(&mut self, prefixes: bool) {
        self.flush_text();
        let trailing = self.trailing_line_endings[usize::from(prefixes)][self.index];
        let mut end = trailing.unwrap_or(self.index);
        // An item whose last construct took its line ending (an unclosed fence) ends
        // after the next item's prefix, as mdast-util-from-markdown's `prepareList` does
        let next = self.index + 1;
        if trailing.is_none()
            && self.events[self.index].name == Name::LIST_ITEM
            && self.events.get(next).is_some_and(|event| {
                event.kind == event::Kind::Enter && event.name == Name::LIST_ITEM
            })
            && let Some(offset) = self.events[next..].iter().position(|event| {
                event.kind == event::Kind::Exit && event.name == Name::LIST_ITEM_PREFIX
            })
        {
            end = next + offset;
        }
        let end = to_point(&self.events[end].point);
        self.set_tail_end(end);
        self.attach_tail();
    }

    fn set_tail_end(&mut self, end: Point) {
        if let Some(Position::Span { end: node_end, .. }) = self.tail_mut().position_mut()
        {
            *node_end = end;
        }
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

    /// Append `addition` to the open text node: borrowing the source while pieces are
    /// adjacent in it, building the value in the arena once they are not.
    pub(crate) fn append_text(&mut self, addition: &'a str) {
        if let Some(builder) = self.text_builder.as_mut() {
            builder.push_str(addition);
            return;
        }
        let Some(Node::Text(text)) = self.stack.last() else {
            return;
        };
        let value = text.value;
        let joined = if value.is_empty() {
            Some(addition)
        } else {
            self.join_adjacent(value, addition)
        };
        match joined {
            Some(joined) => {
                if let Some(Node::Text(text)) = self.stack.last_mut() {
                    text.value = joined;
                }
            }
            None => {
                let mut builder = ArenaString::with_capacity_in(
                    value.len() + addition.len(),
                    self.alloc,
                );
                builder.push_str(value);
                builder.push_str(addition);
                self.text_builder = Some(builder);
            }
        }
    }

    // Write the text being built into its node: the innermost one, or that one's last child
    fn flush_text(&mut self) {
        let Some(builder) = self.text_builder.take() else {
            return;
        };
        let value = builder.into_bump_str();
        let node = match self.stack.last_mut() {
            Some(Node::Text(text)) => Some(text),
            Some(parent) => match parent
                .children_mut()
                .and_then(|children| children.last_mut())
            {
                Some(Node::Text(text)) => Some(text),
                _ => None,
            },
            None => None,
        };
        if let Some(text) = node {
            text.value = value;
        }
    }

    /// The node the innermost open node sits in.
    pub(crate) fn penultimate_mut(&mut self) -> Option<&mut Node<'a>> {
        let index = self.stack.len().checked_sub(2)?;
        self.stack.get_mut(index)
    }

    /// Collect the text that follows into a buffer, until `resume`.
    pub(crate) fn buffer(&mut self) {
        self.flush_text();
        self.stack.push(Node::Paragraph(Paragraph {
            children: ArenaVec::new_in(self.alloc),
            position: None,
        }));
    }

    /// Close the buffer and return it, with the phrasing collected in it.
    pub(crate) fn resume_node(&mut self) -> Option<Node<'a>> {
        self.flush_text();
        self.stack.pop()
    }

    /// The text of a node, as mdast-util-to-string reads it: values, image alts and the
    /// text of children, walked without recursing.
    pub(crate) fn text_of(&self, node: &Node<'a>) -> &'a str {
        let mut pieces = vec![];
        let mut stack = vec![node];
        while let Some(node) = stack.pop() {
            let text = match node {
                Node::Text(text) => Some(text.value),
                Node::InlineCode(code) => Some(code.value),
                Node::Code(code) => Some(code.value),
                Node::Image(image) => image.alt,
                Node::ImageReference(image) => image.alt,
                _ => None,
            };
            pieces.extend(text);
            if let Some(children) = node.children() {
                stack.extend(children.iter().rev());
            }
        }
        match pieces.as_slice() {
            [] => "",
            [piece] => piece,
            _ => {
                let length = pieces.iter().map(|piece| piece.len()).sum();
                let mut value = ArenaString::with_capacity_in(length, self.alloc);
                pieces.iter().for_each(|piece| value.push_str(piece));
                value.into_bump_str()
            }
        }
    }

    /// The source of the token whose exit is the current event, decoded and allocated.
    pub(crate) fn alloc_str(&self, value: &str) -> &'a str {
        self.alloc.alloc_str(value)
    }

    /// Close the buffer and return the text collected in it.
    pub(crate) fn resume(&mut self) -> &'a str {
        match self.resume_node() {
            Some(buffer) => self.text_of(&buffer),
            None => "",
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
        self.slice_at(self.index)
    }

    /// The source text of the token whose exit is at `exit`.
    pub(crate) fn slice_at(&self, exit: usize) -> &'a str {
        let end = &self.events[exit].point;
        let start = &self.events[self.enters[exit]].point;
        let slice = Slice::from_points(self.source.as_bytes(), start, end);
        let text = std::str::from_utf8(slice.bytes).unwrap_or_default();
        // Prefixes of lines after the first are not part of the token's text
        let first = self
            .prefixes
            .partition_point(|prefix| prefix.0 <= start.index);
        let prefixed = self.prefixes[first..]
            .iter()
            .take_while(|prefix| prefix.0 < end.index)
            .count();
        let clean = slice.before == 0 && slice.after == 0 && prefixed == 0;
        if clean && !text.contains('\0') {
            return text;
        }
        let mut value = ArenaString::with_capacity_in(slice.len(), self.alloc);
        // A token that starts or ends inside a tab has the tab's columns there as spaces
        value.extend(std::iter::repeat_n(' ', slice.before));
        let offset = start.index + usize::from(start.vs > 0);
        let mut at = 0;
        for (line_start, content_start) in &self.prefixes[first..first + prefixed] {
            let from = line_start.saturating_sub(offset).min(text.len());
            value.push_str(&text[at.min(from)..from]);
            at = content_start.saturating_sub(offset).min(text.len());
        }
        value.push_str(&text[at.min(text.len())..]);
        value.extend(std::iter::repeat_n(' ', slice.after));
        // micromark reads NUL as the replacement character
        if value.contains('\0') {
            return self.alloc.alloc_str(&value.replace('\0', "\u{FFFD}"));
        }
        value.into_bump_str()
    }

    // `value` followed by `addition` when they are adjacent in the source
    fn join_adjacent(&self, value: &'a str, addition: &'a str) -> Option<&'a str> {
        let (start, next) = (self.offset_of(value)?, self.offset_of(addition)?);
        (start + value.len() == next)
            .then(|| self.source.get(start..next + addition.len()))
            .flatten()
    }

    fn offset_of(&self, value: &str) -> Option<usize> {
        let base = self.source.as_ptr() as usize;
        let at = value.as_ptr() as usize;
        (at >= base && at + value.len() <= base + self.source.len()).then(|| at - base)
    }
}

fn find_enters(events: &[Event]) -> Vec<usize> {
    let mut enters = vec![0; events.len()];
    let mut open = vec![];
    for (index, event) in events.iter().enumerate() {
        match event.kind {
            event::Kind::Enter => {
                open.push(index);
                enters[index] = index;
            }
            event::Kind::Exit => enters[index] = open.pop().unwrap_or(index),
        }
    }
    enters
}

pub(crate) fn to_point(point: &event::Point) -> Point {
    Point::new(point.index, point.vs)
}
