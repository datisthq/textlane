use crate::ArenaVec;
use crate::models::position::Position;

/// A hast node.
#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Root(Root<'a>),
    Element(Element<'a>),
    Text(Text<'a>),
    Comment(Comment<'a>),
    Doctype(Doctype<'a>),
    Raw(Raw<'a>),
}

/// The document or fragment.
#[derive(Debug, PartialEq)]
pub struct Root<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
    /// `data.quirksMode`, set by the HTML parser: whether the document is in quirks mode.
    pub quirks_mode: Option<bool>,
}

/// An element.
#[derive(Debug, PartialEq)]
pub struct Element<'a> {
    pub tag_name: &'a str,
    pub properties: ArenaVec<'a, Property<'a>>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
    /// A `template`'s contents, which hast keeps out of its children.
    pub content: Option<&'a Root<'a>>,
}

/// One property of an element, by its hast name (`className`, not `class`).
#[derive(Debug, PartialEq)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: PropertyValue<'a>,
}

/// A property value as hast allows it.
#[derive(Debug, PartialEq)]
pub enum PropertyValue<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    String(&'a str),
    List(ArenaVec<'a, PropertyItem<'a>>),
}

/// One item of a list property value, such as one class name.
#[derive(Debug, PartialEq)]
pub enum PropertyItem<'a> {
    Number(f64),
    String(&'a str),
}

/// Text.
#[derive(Debug, PartialEq)]
pub struct Text<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A comment.
#[derive(Debug, PartialEq)]
pub struct Comment<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// Raw HTML, written as it is only when dangerous HTML is allowed.
#[derive(Debug, PartialEq)]
pub struct Raw<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A doctype.
#[derive(Debug, PartialEq)]
pub struct Doctype<'a> {
    pub position: Option<Position<'a>>,
}

impl<'a> Node<'a> {
    pub fn position(&self) -> Option<&Position<'a>> {
        match self {
            Node::Root(node) => node.position.as_ref(),
            Node::Element(node) => node.position.as_ref(),
            Node::Text(node) => node.position.as_ref(),
            Node::Comment(node) => node.position.as_ref(),
            Node::Doctype(node) => node.position.as_ref(),
            Node::Raw(node) => node.position.as_ref(),
        }
    }
}
