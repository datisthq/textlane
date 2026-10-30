use textlane_core::mdast::{
    Blockquote, Break, Code, ContainerDirective, DefList, DefListDescription,
    DefListTerm, Definition, Delete, Emphasis, FootnoteDefinition, FootnoteReference,
    Heading, Html, Image, ImageReference, InlineCode, InlineMath, LeafDirective, Link,
    LinkReference, List, ListItem, Literal, Math, MdxExpression, MdxJsxElement, MdxjsEsm,
    Node, Paragraph, Root, Strong, Table, TableCell, TableRow, Text, TextDirective,
    ThematicBreak, Toml, Unknown, Yaml,
};
use textlane_core::{Allocator, ArenaVec, Position};

/// How much of a tree `copy_nodes` keeps.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CopyMode {
    /// mdast-util-toc's `all`: positions dropped, footnote calls dropped, links and link
    /// references unwrapped into their children.
    Phrasing,
    /// Every node as it is.
    Whole,
}

/// A deep copy of `nodes` in the arena; `data` is shared, since nothing mutates it in place.
pub(crate) fn copy_nodes<'a>(
    nodes: &[Node<'a>],
    mode: CopyMode,
    alloc: &'a Allocator,
) -> ArenaVec<'a, Node<'a>> {
    // Each open parent: its remaining children, the copies made so far and its own copy,
    // which `None` leaves to the frame below (the top, or a link being unwrapped)
    let mut frames = vec![Frame {
        children: nodes.iter(),
        copies: ArenaVec::new_in(alloc),
        shell: None,
    }];
    loop {
        let Some(frame) = frames.last_mut() else {
            return ArenaVec::new_in(alloc);
        };
        let Some(node) = frame.children.next() else {
            let Some(done) = frames.pop() else {
                return ArenaVec::new_in(alloc);
            };
            let Some(below) = frames.last_mut() else {
                return done.copies;
            };
            match done.shell {
                Some(mut shell) => {
                    if let Some(children) = shell.children_mut() {
                        *children = done.copies;
                    }
                    below.copies.push(shell);
                }
                None => below.copies.extend(done.copies),
            }
            continue;
        };
        if mode == CopyMode::Phrasing {
            match node {
                Node::FootnoteReference(_) => continue,
                Node::Link(Link { children, .. })
                | Node::LinkReference(LinkReference { children, .. }) => {
                    frames.push(Frame {
                        children: children.iter(),
                        copies: ArenaVec::new_in(alloc),
                        shell: None,
                    });
                    continue;
                }
                _ => {}
            }
        }
        let keep = mode == CopyMode::Whole;
        let copy = copy_shallow(node, keep, alloc);
        match node.children() {
            Some(children) if !children.is_empty() => frames.push(Frame {
                children: children.iter(),
                copies: ArenaVec::with_capacity_in(children.len(), alloc),
                shell: Some(copy),
            }),
            _ => frame.copies.push(copy),
        }
    }
}

struct Frame<'n, 'a> {
    children: std::slice::Iter<'n, Node<'a>>,
    copies: ArenaVec<'a, Node<'a>>,
    shell: Option<Node<'a>>,
}

// The node without its children, its position kept only when `keep` is set
fn copy_shallow<'a>(node: &Node<'a>, keep: bool, alloc: &'a Allocator) -> Node<'a> {
    let position: Option<Position<'a>> =
        if keep { node.position().copied() } else { None };
    let data = node.data();
    let none = || ArenaVec::new_in(alloc);
    match node {
        Node::Root(_) => Node::Root(Root {
            children: none(),
            position,
            data,
        }),
        Node::Paragraph(_) => Node::Paragraph(Paragraph {
            children: none(),
            position,
            data,
        }),
        Node::Heading(node) => Node::Heading(Heading {
            depth: node.depth,
            children: none(),
            position,
            data,
        }),
        Node::ThematicBreak(_) => Node::ThematicBreak(ThematicBreak { position, data }),
        Node::Text(node) => Node::Text(Text {
            value: node.value,
            position,
            data,
        }),
        Node::Emphasis(_) => Node::Emphasis(Emphasis {
            children: none(),
            position,
            data,
        }),
        Node::Strong(_) => Node::Strong(Strong {
            children: none(),
            position,
            data,
        }),
        Node::Break(_) => Node::Break(Break { position, data }),
        Node::Code(node) => Node::Code(Code {
            lang: node.lang,
            meta: node.meta,
            value: node.value,
            position,
            data,
        }),
        Node::InlineCode(node) => Node::InlineCode(InlineCode {
            value: node.value,
            position,
            data,
        }),
        Node::Blockquote(_) => Node::Blockquote(Blockquote {
            children: none(),
            position,
            data,
        }),
        Node::List(node) => Node::List(List {
            ordered: node.ordered,
            start: node.start,
            spread: node.spread,
            children: none(),
            position,
            data,
        }),
        Node::ListItem(node) => Node::ListItem(ListItem {
            spread: node.spread,
            checked: node.checked,
            children: none(),
            position,
            data,
        }),
        Node::Link(node) => Node::Link(Link {
            url: node.url,
            title: node.title,
            children: none(),
            position,
            data,
        }),
        Node::Image(node) => Node::Image(Image {
            url: node.url,
            title: node.title,
            alt: node.alt,
            position,
            data,
        }),
        Node::LinkReference(node) => Node::LinkReference(LinkReference {
            identifier: node.identifier,
            label: node.label,
            reference_type: node.reference_type,
            children: none(),
            position,
            data,
        }),
        Node::ImageReference(node) => Node::ImageReference(ImageReference {
            identifier: node.identifier,
            label: node.label,
            reference_type: node.reference_type,
            alt: node.alt,
            position,
            data,
        }),
        Node::Definition(node) => Node::Definition(Definition {
            identifier: node.identifier,
            label: node.label,
            url: node.url,
            title: node.title,
            position,
            data,
        }),
        Node::Html(node) => Node::Html(Html {
            value: node.value,
            position,
            data,
        }),
        Node::Delete(_) => Node::Delete(Delete {
            children: none(),
            position,
            data,
        }),
        Node::Table(node) => {
            let mut align = ArenaVec::with_capacity_in(node.align.len(), alloc);
            align.extend(node.align.iter().copied());
            Node::Table(Table {
                align,
                children: none(),
                position,
                data,
            })
        }
        Node::TableRow(_) => Node::TableRow(TableRow {
            children: none(),
            position,
            data,
        }),
        Node::TableCell(_) => Node::TableCell(TableCell {
            children: none(),
            position,
            data,
        }),
        Node::FootnoteDefinition(node) => Node::FootnoteDefinition(FootnoteDefinition {
            identifier: node.identifier,
            label: node.label,
            children: none(),
            position,
            data,
        }),
        Node::FootnoteReference(node) => Node::FootnoteReference(FootnoteReference {
            identifier: node.identifier,
            label: node.label,
            position,
            data,
        }),
        Node::Math(node) => Node::Math(Math {
            meta: node.meta,
            value: node.value,
            position,
            data,
        }),
        Node::InlineMath(node) => Node::InlineMath(InlineMath {
            value: node.value,
            position,
            data,
        }),
        Node::ContainerDirective(node) => Node::ContainerDirective(ContainerDirective {
            name: node.name,
            attributes: copy_pairs(&node.attributes, alloc),
            label: node.label,
            children: none(),
            position,
            data,
        }),
        Node::LeafDirective(node) => Node::LeafDirective(LeafDirective {
            name: node.name,
            attributes: copy_pairs(&node.attributes, alloc),
            children: none(),
            position,
            data,
        }),
        Node::TextDirective(node) => Node::TextDirective(TextDirective {
            name: node.name,
            attributes: copy_pairs(&node.attributes, alloc),
            children: none(),
            position,
            data,
        }),
        Node::Yaml(node) => Node::Yaml(Yaml {
            value: node.value,
            position,
            data,
        }),
        Node::Toml(node) => Node::Toml(Toml {
            value: node.value,
            position,
            data,
        }),
        Node::Literal(node) => Node::Literal(Literal {
            kind: node.kind,
            value: node.value,
            position,
            data,
        }),
        Node::Unknown(node) => Node::Unknown(Unknown {
            kind: node.kind,
            fields: node.fields,
            children: node.children.as_ref().map(|_| none()),
            position,
            data,
        }),
        Node::DefList(_) => Node::DefList(DefList {
            children: none(),
            position,
            data,
        }),
        Node::DefListTerm(_) => Node::DefListTerm(DefListTerm {
            children: none(),
            position,
            data,
        }),
        Node::DefListDescription(node) => Node::DefListDescription(DefListDescription {
            spread: node.spread,
            children: none(),
            position,
            data,
        }),
        Node::MdxJsxFlowElement(node) => {
            Node::MdxJsxFlowElement(copy_element(node, position, alloc))
        }
        Node::MdxJsxTextElement(node) => {
            Node::MdxJsxTextElement(copy_element(node, position, alloc))
        }
        Node::MdxFlowExpression(node) => Node::MdxFlowExpression(MdxExpression {
            value: node.value,
            position,
            data,
        }),
        Node::MdxTextExpression(node) => Node::MdxTextExpression(MdxExpression {
            value: node.value,
            position,
            data,
        }),
        Node::MdxjsEsm(node) => Node::MdxjsEsm(MdxjsEsm {
            value: node.value,
            position,
            data,
        }),
    }
}

fn copy_pairs<'a>(
    pairs: &ArenaVec<'a, (&'a str, &'a str)>,
    alloc: &'a Allocator,
) -> ArenaVec<'a, (&'a str, &'a str)> {
    let mut copy = ArenaVec::with_capacity_in(pairs.len(), alloc);
    copy.extend(pairs.iter().copied());
    copy
}

// structuredClone keeps the attributes' own positions: only the node's is dropped
fn copy_element<'a>(
    node: &MdxJsxElement<'a, Node<'a>>,
    position: Option<Position<'a>>,
    alloc: &'a Allocator,
) -> MdxJsxElement<'a, Node<'a>> {
    let mut attributes = ArenaVec::with_capacity_in(node.attributes.len(), alloc);
    attributes.extend(node.attributes.iter().cloned());
    MdxJsxElement {
        name: node.name,
        attributes,
        children: ArenaVec::new_in(alloc),
        position,
        data: node.data,
    }
}
