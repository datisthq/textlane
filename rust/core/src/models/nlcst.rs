use crate::ArenaVec;
use crate::models::data::Data;
use crate::models::position::Position;

/// An nlcst node: natural language as paragraphs, sentences and words.
#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Root(Parent<'a>),
    Paragraph(Parent<'a>),
    Sentence(Parent<'a>),
    Word(Parent<'a>),
    Text(Literal<'a>),
    Symbol(Literal<'a>),
    Punctuation(Literal<'a>),
    WhiteSpace(Literal<'a>),
    Source(Literal<'a>),
    Unknown(Unknown<'a>),
}

/// A node with children: a root, paragraph, sentence or word.
#[derive(Debug, PartialEq)]
pub struct Parent<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// A node with a value: text, a symbol, punctuation, white space or source.
#[derive(Debug, PartialEq)]
pub struct Literal<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// A node of a type nlcst does not define.
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
    /// The node's nlcst `type`, as JSON writes it (`WordNode`).
    pub fn kind(&self) -> &'a str {
        match self {
            Node::Root(_) => "RootNode",
            Node::Paragraph(_) => "ParagraphNode",
            Node::Sentence(_) => "SentenceNode",
            Node::Word(_) => "WordNode",
            Node::Text(_) => "TextNode",
            Node::Symbol(_) => "SymbolNode",
            Node::Punctuation(_) => "PunctuationNode",
            Node::WhiteSpace(_) => "WhiteSpaceNode",
            Node::Source(_) => "SourceNode",
            Node::Unknown(node) => node.kind,
        }
    }

    /// The node's children, when it is a parent.
    pub fn children(&self) -> Option<&ArenaVec<'a, Node<'a>>> {
        match self {
            Node::Root(node)
            | Node::Paragraph(node)
            | Node::Sentence(node)
            | Node::Word(node) => Some(&node.children),
            Node::Unknown(node) => node.children.as_ref(),
            _ => None,
        }
    }

    /// The node's value, when it is a literal.
    pub fn value(&self) -> Option<&'a str> {
        match self {
            Node::Text(node)
            | Node::Symbol(node)
            | Node::Punctuation(node)
            | Node::WhiteSpace(node)
            | Node::Source(node) => Some(node.value),
            _ => None,
        }
    }

    /// The node's unist `data`, when it carries any.
    pub fn data(&self) -> Option<&'a Data<'a>> {
        match self {
            Node::Root(node)
            | Node::Paragraph(node)
            | Node::Sentence(node)
            | Node::Word(node) => node.data,
            Node::Text(node)
            | Node::Symbol(node)
            | Node::Punctuation(node)
            | Node::WhiteSpace(node)
            | Node::Source(node) => node.data,
            Node::Unknown(node) => node.data,
        }
    }

    pub fn position(&self) -> Option<&Position<'a>> {
        match self {
            Node::Root(node)
            | Node::Paragraph(node)
            | Node::Sentence(node)
            | Node::Word(node) => node.position.as_ref(),
            Node::Text(node)
            | Node::Symbol(node)
            | Node::Punctuation(node)
            | Node::WhiteSpace(node)
            | Node::Source(node) => node.position.as_ref(),
            Node::Unknown(node) => node.position.as_ref(),
        }
    }
}
