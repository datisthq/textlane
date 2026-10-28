use crate::helpers::tag::TagId;
use crate::models::namespace::Namespace;

/// A byte span of the input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// What parse5's tokenizer emits, with byte spans instead of its line and column locations.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Character(CharacterToken),
    StartTag(TagToken),
    EndTag(TagToken),
    Comment(CommentToken),
    Doctype(DoctypeToken),
    Eof(Span),
}

/// Which run of characters a character token is: parse5 splits them so the tree builder
/// can tell whitespace and NUL apart cheaply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CharacterKind {
    Character,
    Null,
    Whitespace,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CharacterToken {
    pub(crate) kind: CharacterKind,
    pub(crate) chars: String,
    pub(crate) span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagToken {
    pub(crate) tag_name: String,
    pub(crate) tag_id: TagId,
    pub(crate) self_closing: bool,
    pub(crate) ack_self_closing: bool,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) span: Span,
}

/// An attribute; the tree builder sets `namespace` and `prefix` on foreign elements.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Attribute {
    pub(crate) name: String,
    pub(crate) namespace: Option<Namespace>,
    pub(crate) prefix: Option<String>,
    pub(crate) value: String,
    pub(crate) span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CommentToken {
    pub(crate) data: String,
    pub(crate) span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DoctypeToken {
    pub(crate) name: Option<String>,
    pub(crate) force_quirks: bool,
    pub(crate) public_id: Option<String>,
    pub(crate) system_id: Option<String>,
    pub(crate) span: Span,
}

impl TagToken {
    /// The value of the last attribute named `name`, as parse5's `getTokenAttr`.
    pub(crate) fn find_attribute(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .rev()
            .find(|attribute| attribute.name == name)
            .map(|attribute| attribute.value.as_str())
    }
}
