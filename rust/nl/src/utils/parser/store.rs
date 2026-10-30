use std::collections::VecDeque;

use textlane_core::nlcst::{Literal, Node, Parent};
use textlane_core::{
    Allocator, ArenaVec, Data, Point, Position, UnistPoint, UnistPosition,
};

/// The type of a node in the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Root,
    Paragraph,
    Sentence,
    Word,
    Text,
    Symbol,
    Punctuation,
    WhiteSpace,
    Source,
    /// A node of a type nlcst does not define, kept whole.
    Foreign,
}

/// One end of a position: a byte point of the source, or a unist point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Place {
    Byte(Point),
    Unist(UnistPoint),
}

/// A position whose ends parse-latin may set one at a time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: Option<Place>,
    pub(crate) end: Option<Place>,
}

/// A node being built; children are ids, so moving a node between parents moves an id.
pub(crate) struct Item<'a> {
    pub(crate) kind: Kind,
    pub(crate) value: &'a str,
    pub(crate) children: VecDeque<usize>,
    pub(crate) position: Option<Span>,
    pub(crate) data: Option<&'a Data<'a>>,
    pub(crate) foreign: Option<Box<Node<'a>>>,
}

/// Every node of one parse, by id: parse-latin's plugins splice children in place, which
/// on arena vectors would cost a shift per splice.
#[derive(Default)]
pub(crate) struct Store<'a> {
    pub(crate) items: Vec<Item<'a>>,
}

impl<'a> Store<'a> {
    pub(crate) fn add(
        &mut self,
        kind: Kind,
        value: &'a str,
        position: Option<Span>,
    ) -> usize {
        self.items.push(Item {
            kind,
            value,
            children: VecDeque::new(),
            position,
            data: None,
            foreign: None,
        });
        self.items.len() - 1
    }

    pub(crate) fn add_parent(
        &mut self,
        kind: Kind,
        children: VecDeque<usize>,
        position: Option<Span>,
    ) -> usize {
        let id = self.add(kind, "", position);
        self.items[id].children = children;
        id
    }

    pub(crate) fn kind(&self, id: usize) -> Kind {
        self.items[id].kind
    }

    /// Whether the node has children, as `'children' in node` asks.
    pub(crate) fn is_parent(&self, id: usize) -> bool {
        matches!(
            self.items[id].kind,
            Kind::Root | Kind::Paragraph | Kind::Sentence | Kind::Word
        )
    }

    pub(crate) fn is_word(&self, id: usize) -> bool {
        self.items[id].kind == Kind::Word
    }

    pub(crate) fn is_mark(&self, id: usize) -> bool {
        matches!(self.items[id].kind, Kind::Symbol | Kind::Punctuation)
    }

    pub(crate) fn value(&self, id: usize) -> &'a str {
        self.items[id].value
    }

    pub(crate) fn children(&self, id: usize) -> &VecDeque<usize> {
        &self.items[id].children
    }

    pub(crate) fn children_mut(&mut self, id: usize) -> &mut VecDeque<usize> {
        &mut self.items[id].children
    }

    /// Move every child of `from` to the end of `to`.
    pub(crate) fn append_children(&mut self, to: usize, from: usize) {
        let moved = std::mem::take(&mut self.items[from].children);
        self.items[to].children.extend(moved);
    }

    /// Set `to`'s start to `from`'s, when both have a position.
    pub(crate) fn copy_start(&mut self, to: usize, from: usize) {
        if let (Some(_), Some(source)) =
            (self.items[to].position, self.items[from].position)
            && let Some(target) = &mut self.items[to].position
        {
            target.start = source.start;
        }
    }

    /// Set `to`'s end to `from`'s, when both have a position.
    pub(crate) fn copy_end(&mut self, to: usize, from: usize) {
        if let (Some(_), Some(source)) =
            (self.items[to].position, self.items[from].position)
            && let Some(target) = &mut self.items[to].position
        {
            target.end = source.end;
        }
    }

    /// The node's text, when it is at most `limit` bytes long: words are compared whole only
    /// against short lists.
    pub(crate) fn short_text(&self, id: usize, limit: usize) -> Option<String> {
        let mut text = String::new();
        let mut stack = vec![id];
        while let Some(id) = stack.pop() {
            let item = &self.items[id];
            if self.is_parent(id) {
                stack.extend(item.children.iter().rev());
            } else {
                text.push_str(self.leaf_value(id));
                if text.len() > limit {
                    return None;
                }
            }
        }
        Some(text)
    }

    /// The first character of the node's text.
    pub(crate) fn first_character(&self, id: usize) -> Option<char> {
        let mut stack = vec![id];
        while let Some(id) = stack.pop() {
            if self.is_parent(id) {
                stack.extend(self.items[id].children.iter().rev());
            } else if let Some(character) = self.leaf_value(id).chars().next() {
                return Some(character);
            }
        }
        None
    }

    /// Whether the node's text is shorter than `limit` UTF-16 code units.
    pub(crate) fn is_shorter(&self, id: usize, limit: usize) -> bool {
        let mut length = 0;
        let mut stack = vec![id];
        while let Some(id) = stack.pop() {
            if self.is_parent(id) {
                stack.extend(self.items[id].children.iter().rev());
            } else {
                length += self.leaf_value(id).encode_utf16().take(limit).count();
                if length >= limit {
                    return false;
                }
            }
        }
        true
    }

    fn leaf_value(&self, id: usize) -> &'a str {
        let item = &self.items[id];
        match &item.foreign {
            Some(node) => {
                crate::actions::nlcst::stringify::read_value(node).unwrap_or("")
            }
            None => item.value,
        }
    }

    /// Copy an arena node and everything in it into the store.
    pub(crate) fn import(&mut self, node: Node<'a>) -> usize {
        let (root, children) = self.import_one(node);
        let mut stack = Vec::new();
        if let Some(children) = children {
            stack.push((root, children.into_iter()));
        }
        while let Some((parent, children)) = stack.last_mut() {
            let parent = *parent;
            match children.next() {
                Some(child) => {
                    let (id, children) = self.import_one(child);
                    self.items[parent].children.push_back(id);
                    if let Some(children) = children {
                        stack.push((id, children.into_iter()));
                    }
                }
                None => {
                    stack.pop();
                }
            }
        }
        root
    }

    fn import_one(&mut self, node: Node<'a>) -> (usize, Option<ArenaVec<'a, Node<'a>>>) {
        let (kind, parent) = match node {
            Node::Root(parent) => (Kind::Root, parent),
            Node::Paragraph(parent) => (Kind::Paragraph, parent),
            Node::Sentence(parent) => (Kind::Sentence, parent),
            Node::Word(parent) => (Kind::Word, parent),
            Node::Text(literal) => {
                return (self.import_literal(Kind::Text, literal), None);
            }
            Node::Symbol(literal) => {
                return (self.import_literal(Kind::Symbol, literal), None);
            }
            Node::Punctuation(literal) => {
                return (self.import_literal(Kind::Punctuation, literal), None);
            }
            Node::WhiteSpace(literal) => {
                return (self.import_literal(Kind::WhiteSpace, literal), None);
            }
            Node::Source(literal) => {
                return (self.import_literal(Kind::Source, literal), None);
            }
            Node::Unknown(_) => {
                let id = self.add(Kind::Foreign, "", None);
                self.items[id].foreign = Some(Box::new(node));
                return (id, None);
            }
        };
        let id = self.add(kind, "", parent.position.map(read_position));
        self.items[id].data = parent.data;
        (id, Some(parent.children))
    }

    fn import_literal(&mut self, kind: Kind, literal: Literal<'a>) -> usize {
        let id = self.add(kind, literal.value, literal.position.map(read_position));
        self.items[id].data = literal.data;
        id
    }

    /// Build the arena node of `id` and everything in it.
    pub(crate) fn export(&mut self, id: usize, alloc: &'a Allocator) -> Node<'a> {
        let mut stack = vec![(id, 0, ArenaVec::new_in(alloc))];
        loop {
            let Some((id, index, _)) = stack.last_mut() else {
                unreachable!("the loop returns once the stack empties")
            };
            let next = self.items[*id].children.get(*index).copied();
            if let Some(child) = next.filter(|_| self.is_parent(*id)) {
                *index += 1;
                if self.is_parent(child) {
                    let capacity = self.items[child].children.len();
                    stack.push((child, 0, ArenaVec::with_capacity_in(capacity, alloc)));
                } else {
                    let node = self.export_one(child, ArenaVec::new_in(alloc), alloc);
                    if let Some((_, _, children)) = stack.last_mut() {
                        children.push(node);
                    }
                }
                continue;
            }
            let Some((id, _, children)) = stack.pop() else {
                unreachable!("the stack holds the node being built")
            };
            let node = self.export_one(id, children, alloc);
            match stack.last_mut() {
                Some((_, _, siblings)) => siblings.push(node),
                None => return node,
            }
        }
    }

    fn export_one(
        &mut self,
        id: usize,
        children: ArenaVec<'a, Node<'a>>,
        alloc: &'a Allocator,
    ) -> Node<'a> {
        let item = &mut self.items[id];
        if let Some(node) = item.foreign.take() {
            return *node;
        }
        let position = item.position.and_then(|span| write_position(span, alloc));
        let (data, value) = (item.data, item.value);
        let parent = || Parent {
            children,
            position,
            data,
        };
        let literal = || Literal {
            value,
            position,
            data,
        };
        match item.kind {
            Kind::Root => Node::Root(parent()),
            Kind::Paragraph => Node::Paragraph(parent()),
            Kind::Sentence => Node::Sentence(parent()),
            Kind::Word => Node::Word(parent()),
            Kind::Symbol => Node::Symbol(literal()),
            Kind::Punctuation => Node::Punctuation(literal()),
            Kind::WhiteSpace => Node::WhiteSpace(literal()),
            Kind::Source => Node::Source(literal()),
            Kind::Text | Kind::Foreign => Node::Text(literal()),
        }
    }
}

/// The span of an arena position.
pub(crate) fn read_position(position: Position<'_>) -> Span {
    match position {
        Position::Span { start, end } => Span {
            start: Some(Place::Byte(start)),
            end: Some(Place::Byte(end)),
        },
        Position::Unist(position) => Span {
            start: Some(Place::Unist(position.start)),
            end: Some(Place::Unist(position.end)),
        },
    }
}

// unist positions have both ends; one parse-latin left half set is dropped
fn write_position<'a>(span: Span, alloc: &'a Allocator) -> Option<Position<'a>> {
    match (span.start?, span.end?) {
        (Place::Byte(start), Place::Byte(end)) => Some(Position::span(start, end)),
        (Place::Unist(start), Place::Unist(end)) => {
            Some(Position::Unist(alloc.alloc(UnistPosition { start, end })))
        }
        _ => None,
    }
}
