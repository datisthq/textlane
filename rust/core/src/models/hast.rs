use crate::ArenaVec;
use crate::models::data::Data;
use crate::models::mdx::{MdxExpression, MdxJsxElement, MdxjsEsm};
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
    Unknown(Unknown<'a>),
    MdxJsxFlowElement(MdxJsxElement<'a, Node<'a>>),
    MdxJsxTextElement(MdxJsxElement<'a, Node<'a>>),
    MdxFlowExpression(MdxExpression<'a>),
    MdxTextExpression(MdxExpression<'a>),
    MdxjsEsm(MdxjsEsm<'a>),
}

/// The document or fragment.
#[derive(Debug, PartialEq)]
pub struct Root<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
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
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
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
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// A comment.
#[derive(Debug, PartialEq)]
pub struct Comment<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// Raw HTML, written as it is only when dangerous HTML is allowed.
#[derive(Debug, PartialEq)]
pub struct Raw<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// A doctype.
#[derive(Debug, PartialEq)]
pub struct Doctype<'a> {
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// A node of a type hast does not define, such as one `md.toHtml` passes through.
#[derive(Debug, PartialEq)]
pub struct Unknown<'a> {
    /// The node's `type`.
    pub kind: &'a str,
    /// Its fields besides `type`, `children`, `position` and `data`, in order.
    pub fields: &'a [(&'a str, Data<'a>)],
    /// Its children, when it is a parent.
    pub children: Option<ArenaVec<'a, Node<'a>>>,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

impl<'a> Node<'a> {
    /// The node's unist `data`, when it carries any.
    pub fn data(&self) -> Option<&'a Data<'a>> {
        match self {
            Node::Root(node) => node.data,
            Node::Element(node) => node.data,
            Node::Text(node) => node.data,
            Node::Comment(node) => node.data,
            Node::Doctype(node) => node.data,
            Node::Raw(node) => node.data,
            Node::Unknown(node) => node.data,
            Node::MdxJsxFlowElement(node) | Node::MdxJsxTextElement(node) => node.data,
            Node::MdxFlowExpression(node) | Node::MdxTextExpression(node) => node.data,
            Node::MdxjsEsm(node) => node.data,
        }
    }

    pub fn position(&self) -> Option<&Position<'a>> {
        match self {
            Node::Root(node) => node.position.as_ref(),
            Node::Element(node) => node.position.as_ref(),
            Node::Text(node) => node.position.as_ref(),
            Node::Comment(node) => node.position.as_ref(),
            Node::Doctype(node) => node.position.as_ref(),
            Node::Raw(node) => node.position.as_ref(),
            Node::Unknown(node) => node.position.as_ref(),
            Node::MdxJsxFlowElement(node) | Node::MdxJsxTextElement(node) => {
                node.position.as_ref()
            }
            Node::MdxFlowExpression(node) | Node::MdxTextExpression(node) => {
                node.position.as_ref()
            }
            Node::MdxjsEsm(node) => node.position.as_ref(),
        }
    }
}
