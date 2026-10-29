use std::borrow::Cow;

use textlane_core::mdast::{self, AlignKind};
use textlane_core::{Allocator, ArenaVec, Position, UnistPosition};

// Building recurses once per level; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// An index into the `Store`.
pub(crate) type Id = usize;

/// The mdast a conversion builds, kept flat until it is done: handlers rewrap, split and
/// edit nodes, and remember facts about each subtree that mdast has no field for.
#[derive(Default)]
pub(crate) struct Store<'a> {
    pub(crate) items: Vec<Item<'a>>,
}

pub(crate) struct Item<'a> {
    pub(crate) kind: Kind<'a>,
    pub(crate) children: Vec<Id>,
    pub(crate) position: Option<Position<'a>>,
    /// Whether a descendant is not phrasing: hast-util-to-mdast's `wrapNeeded` of the
    /// children, kept so nested links and deletes are not searched again at each level.
    pub(crate) flow: bool,
    /// Whether the node or a descendant is a link.
    pub(crate) link: bool,
}

#[derive(Clone)]
pub(crate) enum Kind<'a> {
    Root,
    Paragraph,
    Heading(u8),
    ThematicBreak,
    Text(Text<'a>),
    Emphasis,
    Strong,
    Break,
    Code {
        lang: Option<&'a str>,
        value: &'a str,
    },
    InlineCode(&'a str),
    Blockquote,
    List {
        ordered: bool,
        start: Option<u32>,
        spread: bool,
    },
    ListItem {
        spread: bool,
        checked: Option<bool>,
    },
    Link {
        url: &'a str,
        title: Option<&'a str>,
    },
    Image {
        url: &'a str,
        title: Option<&'a str>,
        alt: &'a str,
    },
    Html(&'a str),
    Delete,
    Table(Vec<AlignKind>),
    TableRow,
    /// A cell, with the row and column span the table handler reads off it.
    TableCell(Option<(usize, usize)>),
    /// Flow content already wrapped, spliced into its parent: re-wrapping it at each
    /// ancestor, as upstream's arrays are, is quadratic in how deep flow containers nest.
    Fragment,
}

/// Text, with the quotes `<q>` adds around it kept apart, so nesting them stays linear.
#[derive(Clone)]
pub(crate) struct Text<'a> {
    pub(crate) value: &'a str,
    /// Opening quotes, innermost first.
    pub(crate) open: Vec<char>,
    pub(crate) close: String,
}

/// What a handler returns: nothing, one node, or several.
pub(crate) enum Converted {
    Nothing,
    One(Id),
    Many(Vec<Id>),
}

impl<'a> Text<'a> {
    pub(crate) fn new(value: &'a str) -> Self {
        Self {
            value,
            open: vec![],
            close: String::new(),
        }
    }

    fn write(&self, output: &mut String) {
        output.extend(self.open.iter().rev());
        output.push_str(self.value);
        output.push_str(&self.close);
    }

    fn is_plain(&self) -> bool {
        self.open.is_empty() && self.close.is_empty()
    }

    /// Whether it is only HTML whitespace, as hast-util-whitespace checks.
    pub(crate) fn is_whitespace(&self) -> bool {
        let blank =
            |character: char| matches!(character, ' ' | '\t' | '\n' | '\u{C}' | '\r');
        self.open.iter().all(|character| blank(*character))
            && self.value.chars().all(blank)
            && self.close.chars().all(blank)
    }
}

impl<'a> Kind<'a> {
    /// An empty list item, as list handlers make one.
    pub(crate) fn list_item() -> Self {
        Kind::ListItem {
            spread: false,
            checked: None,
        }
    }

    /// Whether mdast-util-phrasing counts the node as phrasing content.
    pub(crate) fn is_phrasing(&self) -> bool {
        matches!(
            self,
            Kind::Break
                | Kind::Delete
                | Kind::Emphasis
                | Kind::Image { .. }
                | Kind::InlineCode(_)
                | Kind::Link { .. }
                | Kind::Strong
                | Kind::Text(_)
        )
    }

    /// Whether the node has children.
    pub(crate) fn is_parent(&self) -> bool {
        !matches!(
            self,
            Kind::Text(_)
                | Kind::ThematicBreak
                | Kind::Break
                | Kind::Code { .. }
                | Kind::InlineCode(_)
                | Kind::Image { .. }
                | Kind::Html(_)
        )
    }
}

impl<'a> Store<'a> {
    pub(crate) fn add(
        &mut self,
        kind: Kind<'a>,
        children: Vec<Id>,
        position: Option<Position<'a>>,
    ) -> Id {
        self.items.push(Item {
            kind,
            children: vec![],
            position,
            flow: false,
            link: false,
        });
        let id = self.items.len() - 1;
        self.set_children(id, children);
        id
    }

    /// A text node.
    pub(crate) fn add_text(
        &mut self,
        value: &'a str,
        position: Option<Position<'a>>,
    ) -> Id {
        self.add(Kind::Text(Text::new(value)), vec![], position)
    }

    pub(crate) fn set_children(&mut self, id: Id, children: Vec<Id>) {
        let flow = self.is_wrap_needed(&children);
        let link = children.iter().any(|child| self.items[*child].link);
        let item = &mut self.items[id];
        item.flow = flow;
        item.link = link || matches!(item.kind, Kind::Link { .. });
        item.children = children;
    }

    /// Nodes with every fragment among them replaced by its content.
    pub(crate) fn expand<'i>(&self, nodes: &'i [Id]) -> Cow<'i, [Id]> {
        let is_fragment = |node: &Id| matches!(self.items[*node].kind, Kind::Fragment);
        if !nodes.iter().any(is_fragment) {
            return Cow::Borrowed(nodes);
        }
        let mut expanded = vec![];
        let mut stack: Vec<&[Id]> = vec![nodes];
        while let Some(nodes) = stack.last_mut() {
            let Some((node, rest)) = nodes.split_first() else {
                stack.pop();
                continue;
            };
            *nodes = rest;
            if is_fragment(node) {
                stack.push(&self.items[*node].children);
            } else {
                expanded.push(*node);
            }
        }
        Cow::Owned(expanded)
    }

    pub(crate) fn take_children(&mut self, id: Id) -> Vec<Id> {
        std::mem::take(&mut self.items[id].children)
    }

    /// A copy of a node without its children, as `structuredClone` makes one.
    pub(crate) fn clone_without_children(&mut self, id: Id, children: Vec<Id>) -> Id {
        let item = &self.items[id];
        let (kind, position) = (item.kind.clone(), item.position);
        self.add(kind, children, position)
    }

    /// Whether some node is not phrasing or holds one that is not, as `wrapNeeded` checks.
    pub(crate) fn is_wrap_needed(&self, nodes: &[Id]) -> bool {
        nodes.iter().any(|node| {
            let item = &self.items[*node];
            !item.kind.is_phrasing() || item.flow
        })
    }

    /// Whether several list items are and one of them is spread, as `listItemsSpread` checks.
    pub(crate) fn is_spread(&self, items: &[Id]) -> bool {
        items.len() > 1
            && items.iter().any(|item| {
                matches!(self.items[*item].kind, Kind::ListItem { spread: true, .. })
            })
    }

    pub(crate) fn is_phrasing(&self, id: Id) -> bool {
        self.items[id].kind.is_phrasing()
    }

    /// The text in nodes, as mdast-util-to-string reads it.
    pub(crate) fn read_string(&self, nodes: &[Id]) -> String {
        let mut output = String::new();
        for node in nodes {
            self.write_string(*node, &mut output);
        }
        output
    }

    fn write_string(&self, id: Id, output: &mut String) {
        let item = &self.items[id];
        match &item.kind {
            Kind::Text(text) => text.write(output),
            Kind::Code { value, .. } | Kind::InlineCode(value) | Kind::Html(value) => {
                output.push_str(value);
            }
            Kind::Image { alt, .. } if !alt.is_empty() => output.push_str(alt),
            _ => stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
                for child in &item.children {
                    self.write_string(*child, output);
                }
            }),
        }
    }

    /// Turn a node into mdast, merging and trimming text as hast-util-to-mdast's last pass
    /// does.
    pub(crate) fn build(&self, id: Id, alloc: &'a Allocator) -> mdast::Node<'a> {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || self.build_node(id, alloc))
    }

    fn build_node(&self, id: Id, alloc: &'a Allocator) -> mdast::Node<'a> {
        let item = &self.items[id];
        let position = item.position;
        let children = || self.build_children(&item.kind, &item.children, alloc);
        match &item.kind {
            Kind::Root => mdast::Node::Root(mdast::Root {
                children: children(),
                position,
                data: None,
            }),
            Kind::Paragraph => mdast::Node::Paragraph(mdast::Paragraph {
                children: children(),
                position,
                data: None,
            }),
            Kind::Heading(depth) => mdast::Node::Heading(mdast::Heading {
                depth: *depth,
                children: children(),
                position,
                data: None,
            }),
            Kind::ThematicBreak => mdast::Node::ThematicBreak(mdast::ThematicBreak {
                position,
                data: None,
            }),
            Kind::Text(text) => {
                let value = if text.is_plain() {
                    text.value
                } else {
                    let mut value = String::new();
                    text.write(&mut value);
                    alloc.alloc_str(&value)
                };
                mdast::Node::Text(mdast::Text {
                    value,
                    position,
                    data: None,
                })
            }
            Kind::Emphasis => mdast::Node::Emphasis(mdast::Emphasis {
                children: children(),
                position,
                data: None,
            }),
            Kind::Strong => mdast::Node::Strong(mdast::Strong {
                children: children(),
                position,
                data: None,
            }),
            Kind::Break => mdast::Node::Break(mdast::Break {
                position,
                data: None,
            }),
            Kind::Code { lang, value } => mdast::Node::Code(mdast::Code {
                lang: *lang,
                meta: None,
                value,
                position,
                data: None,
            }),
            Kind::InlineCode(value) => mdast::Node::InlineCode(mdast::InlineCode {
                value,
                position,
                data: None,
            }),
            Kind::Blockquote => mdast::Node::Blockquote(mdast::Blockquote {
                children: children(),
                position,
                data: None,
            }),
            Kind::List {
                ordered,
                start,
                spread,
            } => mdast::Node::List(mdast::List {
                ordered: *ordered,
                start: *start,
                spread: Some(*spread),
                children: children(),
                position,
                data: None,
            }),
            Kind::ListItem { spread, checked } => {
                mdast::Node::ListItem(mdast::ListItem {
                    spread: Some(*spread),
                    checked: *checked,
                    children: children(),
                    position,
                    data: None,
                })
            }
            Kind::Link { url, title } => mdast::Node::Link(mdast::Link {
                url,
                title: *title,
                children: children(),
                position,
                data: None,
            }),
            Kind::Image { url, title, alt } => mdast::Node::Image(mdast::Image {
                url,
                title: *title,
                alt: Some(alt),
                position,
                data: None,
            }),
            Kind::Html(value) => mdast::Node::Html(mdast::Html {
                value,
                position,
                data: None,
            }),
            Kind::Delete => mdast::Node::Delete(mdast::Delete {
                children: children(),
                position,
                data: None,
            }),
            Kind::Table(align) => {
                let mut kinds = ArenaVec::with_capacity_in(align.len(), alloc);
                kinds.extend(align.iter().copied());
                mdast::Node::Table(mdast::Table {
                    align: kinds,
                    children: children(),
                    position,
                    data: None,
                })
            }
            Kind::TableRow => mdast::Node::TableRow(mdast::TableRow {
                children: children(),
                position,
                data: None,
            }),
            Kind::TableCell(_) => mdast::Node::TableCell(mdast::TableCell {
                children: children(),
                position,
                data: None,
            }),
            Kind::Fragment => mdast::Node::Root(mdast::Root {
                children: children(),
                position,
                data: None,
            }),
        }
    }

    // Adjacent texts merge into one; the first line ending of each loses the spaces and
    // tabs around it, and in headings, paragraphs and roots texts are trimmed at the edges
    // and dropped when that empties them. Each text is added and reprocessed in turn, as
    // `visit` does, which comes to processing each run of texts once it ends.
    fn build_children(
        &self,
        parent: &Kind<'a>,
        children: &[Id],
        alloc: &'a Allocator,
    ) -> ArenaVec<'a, mdast::Node<'a>> {
        let trims = matches!(parent, Kind::Heading(_) | Kind::Paragraph | Kind::Root);
        let children = self.expand(children);
        let children = &children[..];
        let mut nodes = ArenaVec::with_capacity_in(children.len(), alloc);
        let mut run: Option<Run<'_, 'a>> = None;
        for (index, child) in children.iter().enumerate() {
            let item = &self.items[*child];
            let Kind::Text(text) = &item.kind else {
                if let Some(run) = run.take() {
                    run.finish(trims, false, &mut nodes, alloc);
                }
                nodes.push(self.build(*child, alloc));
                continue;
            };
            let last = index == children.len() - 1;
            let current = run.get_or_insert_with(|| Run {
                texts: vec![],
                index: nodes.len(),
                position: item.position,
                blank: true,
                empty: true,
            });
            if !current.texts.is_empty()
                && let (Some(start), Some(end)) = (current.position, item.position)
            {
                current.position = merge_positions(start, end, alloc);
            }
            current.add(text);
            // At the start, a run of only spaces and tabs is trimmed away before the next
            // text can join it; elsewhere only an empty run is dropped
            let dropped = if trims && current.index == 0 {
                current.blank
            } else {
                current.empty
            };
            if dropped {
                run = None;
            } else if last && let Some(run) = run.take() {
                run.finish(trims, true, &mut nodes, alloc);
            }
        }
        if let Some(run) = run.take() {
            run.finish(trims, false, &mut nodes, alloc);
        }
        nodes
    }
}

// Texts that merge into one
struct Run<'t, 'a> {
    texts: Vec<&'t Text<'a>>,
    index: usize,
    position: Option<Position<'a>>,
    blank: bool,
    empty: bool,
}

impl<'t, 'a> Run<'t, 'a> {
    fn add(&mut self, text: &'t Text<'a>) {
        let blank = |character: char| character == ' ' || character == '\t';
        self.blank = self.blank
            && text.open.iter().all(|character| blank(*character))
            && text.value.chars().all(blank)
            && text.close.chars().all(blank);
        self.empty = self.empty
            && text.open.is_empty()
            && text.value.is_empty()
            && text.close.is_empty();
        self.texts.push(text);
    }

    fn finish(
        self,
        trims: bool,
        last: bool,
        nodes: &mut ArenaVec<'a, mdast::Node<'a>>,
        alloc: &'a Allocator,
    ) {
        let value = match self.texts[..] {
            [text] if text.is_plain() => Cow::Borrowed(text.value),
            _ => {
                let mut output = String::new();
                for text in &self.texts {
                    text.write(&mut output);
                }
                Cow::Owned(output)
            }
        };
        let mut value = collapse_first_line_ending(value);
        if trims {
            if self.index == 0 {
                value = trim_cow(value, |text| text.trim_start_matches([' ', '\t']));
            }
            if last {
                value = trim_cow(value, |text| text.trim_end_matches([' ', '\t']));
            }
        }
        if value.is_empty() {
            return;
        }
        let value = match value {
            Cow::Borrowed(value) => value,
            Cow::Owned(value) => alloc.alloc_str(&value),
        };
        nodes.push(mdast::Node::Text(mdast::Text {
            value,
            position: self.position,
            data: None,
        }));
    }
}

// `value.replace(/[\t ]*(\r?\n|\r)[\t ]*/, '$1')`: only the first line ending
fn collapse_first_line_ending(value: Cow<'_, str>) -> Cow<'_, str> {
    let bytes = value.as_bytes();
    let Some(ending) = bytes.iter().position(|byte| matches!(byte, b'\n' | b'\r')) else {
        return value;
    };
    let is_space = |byte: &u8| matches!(byte, b' ' | b'\t');
    let start = ending
        - bytes[..ending]
            .iter()
            .rev()
            .take_while(|byte| is_space(byte))
            .count();
    let ending_end = if bytes[ending] == b'\r' && bytes.get(ending + 1) == Some(&b'\n') {
        ending + 2
    } else {
        ending + 1
    };
    let end = ending_end
        + bytes[ending_end..]
            .iter()
            .take_while(|byte| is_space(byte))
            .count();
    if start == ending && end == ending_end {
        return value;
    }
    let mut output = String::with_capacity(value.len());
    output.push_str(&value[..start]);
    output.push_str(&value[ending..ending_end]);
    output.push_str(&value[end..]);
    Cow::Owned(output)
}

fn trim_cow<'v>(value: Cow<'v, str>, trim: impl Fn(&str) -> &str) -> Cow<'v, str> {
    match value {
        Cow::Borrowed(value) => Cow::Borrowed(trim(value)),
        Cow::Owned(value) => Cow::Owned(trim(&value).to_string()),
    }
}

// The span from one text's start to another's end; a unist position joins only another
fn merge_positions<'a>(
    start: Position<'a>,
    end: Position<'a>,
    alloc: &'a Allocator,
) -> Option<Position<'a>> {
    match (start, end) {
        (Position::Span { start, .. }, Position::Span { end, .. }) => {
            Some(Position::span(start, end))
        }
        (Position::Unist(start), Position::Unist(end)) => {
            Some(Position::Unist(alloc.alloc(UnistPosition {
                start: start.start,
                end: end.end,
            })))
        }
        _ => Some(start),
    }
}
