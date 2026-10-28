use crate::ArenaVec;
use crate::models::position::Position;

/// An mdast node.
#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Root(Root<'a>),
    Paragraph(Paragraph<'a>),
    Heading(Heading<'a>),
    ThematicBreak(ThematicBreak<'a>),
    Text(Text<'a>),
    Emphasis(Emphasis<'a>),
    Strong(Strong<'a>),
    Break(Break<'a>),
    Code(Code<'a>),
    InlineCode(InlineCode<'a>),
    Blockquote(Blockquote<'a>),
    List(List<'a>),
    ListItem(ListItem<'a>),
}

/// The document.
#[derive(Debug, PartialEq)]
pub struct Root<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A paragraph.
#[derive(Debug, PartialEq)]
pub struct Paragraph<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A heading of rank `depth`, 1 to 6.
#[derive(Debug, PartialEq)]
pub struct Heading<'a> {
    pub depth: u8,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A thematic break.
#[derive(Debug, PartialEq)]
pub struct ThematicBreak<'a> {
    pub position: Option<Position<'a>>,
}

/// Stressed text, `*a*`.
#[derive(Debug, PartialEq)]
pub struct Emphasis<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// Strongly stressed text, `**a**`.
#[derive(Debug, PartialEq)]
pub struct Strong<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A hard line break.
#[derive(Debug, PartialEq)]
pub struct Break<'a> {
    pub position: Option<Position<'a>>,
}

/// A block of code, fenced or indented; `lang` and `meta` come from a fence's info string.
#[derive(Debug, PartialEq)]
pub struct Code<'a> {
    pub lang: Option<&'a str>,
    pub meta: Option<&'a str>,
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A code span.
#[derive(Debug, PartialEq)]
pub struct InlineCode<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A block quote.
#[derive(Debug, PartialEq)]
pub struct Blockquote<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A list: `ordered` from `1.`, starting at `start`; `spread` when its items are separated
/// by blank lines.
#[derive(Debug, PartialEq)]
pub struct List<'a> {
    pub ordered: bool,
    pub start: Option<u32>,
    pub spread: Option<bool>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A list item; `checked` is a GFM task's state.
#[derive(Debug, PartialEq)]
pub struct ListItem<'a> {
    pub spread: Option<bool>,
    pub checked: Option<bool>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// Plain text.
#[derive(Debug, PartialEq)]
pub struct Text<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

impl<'a> Node<'a> {
    /// The node's children, when it is a parent.
    pub fn children(&self) -> Option<&ArenaVec<'a, Node<'a>>> {
        match self {
            Node::Root(node) => Some(&node.children),
            Node::Paragraph(node) => Some(&node.children),
            Node::Heading(node) => Some(&node.children),
            Node::Emphasis(node) => Some(&node.children),
            Node::Strong(node) => Some(&node.children),
            Node::Blockquote(node) => Some(&node.children),
            Node::List(node) => Some(&node.children),
            Node::ListItem(node) => Some(&node.children),
            Node::ThematicBreak(_)
            | Node::Text(_)
            | Node::Break(_)
            | Node::Code(_)
            | Node::InlineCode(_) => None,
        }
    }

    /// The node's children, mutably, when it is a parent.
    pub fn children_mut(&mut self) -> Option<&mut ArenaVec<'a, Node<'a>>> {
        match self {
            Node::Root(node) => Some(&mut node.children),
            Node::Paragraph(node) => Some(&mut node.children),
            Node::Heading(node) => Some(&mut node.children),
            Node::Emphasis(node) => Some(&mut node.children),
            Node::Strong(node) => Some(&mut node.children),
            Node::Blockquote(node) => Some(&mut node.children),
            Node::List(node) => Some(&mut node.children),
            Node::ListItem(node) => Some(&mut node.children),
            Node::ThematicBreak(_)
            | Node::Text(_)
            | Node::Break(_)
            | Node::Code(_)
            | Node::InlineCode(_) => None,
        }
    }

    pub fn position(&self) -> Option<&Position<'a>> {
        match self {
            Node::Root(node) => node.position.as_ref(),
            Node::Paragraph(node) => node.position.as_ref(),
            Node::Heading(node) => node.position.as_ref(),
            Node::ThematicBreak(node) => node.position.as_ref(),
            Node::Text(node) => node.position.as_ref(),
            Node::Emphasis(node) => node.position.as_ref(),
            Node::Strong(node) => node.position.as_ref(),
            Node::Break(node) => node.position.as_ref(),
            Node::Code(node) => node.position.as_ref(),
            Node::InlineCode(node) => node.position.as_ref(),
            Node::Blockquote(node) => node.position.as_ref(),
            Node::List(node) => node.position.as_ref(),
            Node::ListItem(node) => node.position.as_ref(),
        }
    }

    pub fn position_mut(&mut self) -> &mut Option<Position<'a>> {
        match self {
            Node::Root(node) => &mut node.position,
            Node::Paragraph(node) => &mut node.position,
            Node::Heading(node) => &mut node.position,
            Node::ThematicBreak(node) => &mut node.position,
            Node::Text(node) => &mut node.position,
            Node::Emphasis(node) => &mut node.position,
            Node::Strong(node) => &mut node.position,
            Node::Break(node) => &mut node.position,
            Node::Code(node) => &mut node.position,
            Node::InlineCode(node) => &mut node.position,
            Node::Blockquote(node) => &mut node.position,
            Node::List(node) => &mut node.position,
            Node::ListItem(node) => &mut node.position,
        }
    }
}
