use textlane_core::mdast::{Node, Paragraph, ReferenceType, Root};
use textlane_core::{Allocator, ArenaString, ArenaVec, Error, Lines, Point, Position};

use std::collections::HashSet;

use crate::helpers::slice::Slice;
use crate::helpers::spread::{TrailingRun, find_loose, find_trailing, read_trailing};
use crate::models::event::{self, Event};
use crate::models::mdx::MdxCompile;
use crate::models::options::Matter;
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
    /// The attributes of the directive being compiled, as name and decoded value.
    pub(crate) directive_attributes: Vec<(&'a str, String)>,
    trailing: Vec<(usize, TrailingRun)>,
    stack: Vec<Node<'a>>,
    // The children of open nodes, each node's from its `starts` entry on, moved into the
    // arena once it closes: growing arena vectors would leave every outgrown buffer behind
    children: Vec<Node<'a>>,
    starts: Vec<usize>,
    // The enter event of each node opened by a token, as mdast-util-from-markdown's token
    // stack: buffers are not on it, so a node left open inside one is still found
    stack_tokens: Vec<usize>,
    /// The first error compiling found, which ends it.
    pub(crate) error: Option<Error>,
    /// What compiling MDX keeps between events.
    pub(crate) mdx: MdxCompile<'a>,
    /// The kinds of frontmatter the parser recognizes, to tell which one a token is.
    pub(crate) frontmatter: &'e [Matter],
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
            data: None,
        });
        let trailing = find_trailing(events);
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
            loose: find_loose(events, &trailing),
            task_list_check_seen: false,
            directive_attributes: vec![],
            trailing,
            stack: vec![root],
            children: vec![],
            starts: vec![0],
            stack_tokens: vec![],
            error: None,
            mdx: MdxCompile::default(),
            frontmatter: &[],
        }
    }

    pub(crate) fn finish(mut self) -> Result<Node<'a>, Error> {
        if let Some(open) = (self.stack_tokens.iter().rev().copied())
            .find(|open| is_mdx_jsx_tag(self.events[*open].name))
        {
            return Err(self.mdx_unclosed_error(open, None));
        }
        self.flush_text();
        // Events always balance, but attach anything left open rather than lose it
        while self.stack.len() > 1 {
            self.attach_tail();
        }
        match self.pop_node() {
            Some(root) => Ok(root),
            None => unreachable!("the root is never popped"),
        }
    }

    fn push_node(&mut self, node: Node<'a>) {
        self.stack.push(node);
        self.starts.push(self.children.len());
    }

    // Close the innermost open node, its children after any it was given directly
    fn pop_node(&mut self) -> Option<Node<'a>> {
        let mut node = self.stack.pop()?;
        let start = self.starts.pop().unwrap_or(self.children.len());
        if start < self.children.len()
            && let Some(children) = node.children_mut()
        {
            let added = self.children.drain(start..);
            if children.is_empty() {
                let mut exact = ArenaVec::with_capacity_in(added.len(), self.alloc);
                exact.extend(added);
                *children = exact;
            } else {
                children.extend(added);
            }
        }
        Some(node)
    }

    /// The last child of the innermost open node.
    pub(crate) fn last_child_mut(&mut self) -> Option<&mut Node<'a>> {
        let start = self.starts.last().copied().unwrap_or(0);
        if start < self.children.len() {
            return self.children.last_mut();
        }
        self.tail_mut()
            .children_mut()
            .and_then(|children| children.last_mut())
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
        self.push_node(child);
        self.stack_tokens.push(self.index);
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
        let start = self.starts.last().copied().unwrap_or(0);
        let last = if start < self.children.len() {
            self.children.pop()
        } else {
            self.tail_mut()
                .children_mut()
                .and_then(|children| children.pop())
        };
        if let Some(last) = last {
            self.push_node(last);
            self.stack_tokens.push(self.index);
        }
    }

    /// Close the innermost open node at the current event.
    pub(crate) fn tail_pop(&mut self) {
        self.check_mdx_jsx_closed();
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
        self.check_mdx_jsx_closed();
        self.flush_text();
        let trailing =
            read_trailing(&self.trailing, self.index).line_ending[usize::from(prefixes)];
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
        self.stack_tokens.pop();
        if let Some(node) = self.pop_node() {
            self.children.push(node);
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
        if let Some(Node::Text(text)) = self.stack.last_mut() {
            text.value = value;
        } else if let Some(Node::Text(text)) = self.last_child_mut() {
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
        self.push_node(Node::Paragraph(Paragraph {
            children: ArenaVec::new_in(self.alloc),
            position: None,
            data: None,
        }));
    }

    /// Close the buffer and return it, with the phrasing collected in it.
    pub(crate) fn resume_node(&mut self) -> Option<Node<'a>> {
        self.flush_text();
        self.pop_node()
    }

    /// The enter event of the innermost node a token opened.
    pub(crate) fn tail_token(&self) -> Option<usize> {
        self.stack_tokens.last().copied()
    }

    // Closing any node but a JSX element while one is open inside it fails, as
    // mdast-util-mdx-jsx's `onErrorRightIsTag` does
    fn check_mdx_jsx_closed(&mut self) {
        if self.mdx.closing || self.error.is_some() {
            return;
        }
        if let Some(open) = self.tail_token()
            && is_mdx_jsx_tag(self.events[open].name)
        {
            self.error = Some(self.mdx_unclosed_error(open, Some(self.index)));
        }
    }

    /// mdast-util-mdx-jsx's error for an element opened at the tag entered at `open`, still
    /// open where the token exited at `closing` ends, or at the end.
    pub(crate) fn mdx_unclosed_error(
        &self,
        open: usize,
        closing: Option<usize>,
    ) -> Error {
        let tag = match self.mdx.open.last() {
            Some(tag) => serialize_abbreviated_tag(tag.name.as_deref(), tag.close),
            None => "<>".into(),
        };
        let place = match closing {
            Some(closing) => {
                format!(
                    " before the end of `{}`",
                    token_type(self.events[closing].name)
                )
            }
            None => String::new(),
        };
        Error::new(
            "mdast-util-mdx-jsx:end-tag-mismatch",
            format!(
                "Expected a closing tag for `{tag}` ({}){place}",
                self.stringify_span(open, self.find_exit(open))
            ),
        )
    }

    /// The exit event of the token entered at `enter`.
    pub(crate) fn find_exit(&self, enter: usize) -> usize {
        let mut depth = 0;
        for (index, event) in self.events.iter().enumerate().skip(enter) {
            match event.kind {
                event::Kind::Enter => depth += 1,
                event::Kind::Exit => {
                    depth -= 1;
                    if depth == 0 {
                        return index;
                    }
                }
            }
        }
        enter
    }

    /// `line:column` of an event, as unist-util-stringify-position writes a point.
    pub(crate) fn stringify_point(&self, index: usize) -> String {
        let point =
            Lines::new(self.source).to_unist_point(to_point(&self.events[index].point));
        format!("{}:{}", point.line, point.column)
    }

    /// `line:column-line:column` of two events.
    pub(crate) fn stringify_span(&self, start: usize, end: usize) -> String {
        format!(
            "{}-{}",
            self.stringify_point(start),
            self.stringify_point(end)
        )
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
                Node::Math(math) => Some(math.value),
                Node::InlineMath(math) => Some(math.value),
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
        if let Some(last) = self.last_child_mut()
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
        // The input is a `str`: a slice of it needs no validating, only char boundaries
        let text = (self.source)
            .get(slice.start..slice.start + slice.bytes.len())
            .unwrap_or_default();
        // Prefixes of lines after the first are not part of the token's text
        let first = self
            .prefixes
            .partition_point(|prefix| prefix.0 <= start.index as usize);
        let prefixed = self.prefixes[first..]
            .iter()
            .take_while(|prefix| prefix.0 < end.index as usize)
            .count();
        let clean = slice.before == 0 && slice.after == 0 && prefixed == 0;
        if clean && !text.contains('\0') {
            return text;
        }
        let mut value = ArenaString::with_capacity_in(slice.len(), self.alloc);
        // A token that starts or ends inside a tab has the tab's columns there as spaces
        value.extend(std::iter::repeat_n(' ', slice.before));
        let offset = start.index + u32::from(start.vs > 0);
        let mut at = 0;
        for (line_start, content_start) in &self.prefixes[first..first + prefixed] {
            let from = line_start.saturating_sub(offset as usize).min(text.len());
            value.push_str(&text[at.min(from)..from]);
            at = content_start
                .saturating_sub(offset as usize)
                .min(text.len());
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

/// Whether a token is a JSX tag, in flow or text.
pub(crate) fn is_mdx_jsx_tag(name: Name) -> bool {
    name.text() == "mdxJsxFlowTag" || name.text() == "mdxJsxTextTag"
}

/// A tag as mdast-util-mdx-jsx names it in errors: `<a>` or `</a>`.
pub(crate) fn serialize_abbreviated_tag(name: Option<&str>, close: bool) -> String {
    format!("<{}{}>", if close { "/" } else { "" }, name.unwrap_or(""))
}

/// A token's type as micromark names it, which errors show.
pub(crate) fn token_type(name: Name) -> &'static str {
    match name {
        Name::HEADING_ATX => "atxHeading",
        Name::HEADING_SETEXT => "setextHeading",
        _ => name.text(),
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
    Point::new(point.index as usize, point.vs as usize)
}
