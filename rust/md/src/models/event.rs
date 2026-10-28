use crate::models::token::Name;

/// Whether an event opens or closes a token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Enter,
    Exit,
}

/// A place in the input as the tokenizer tracks it: lines and columns count bytes, `vs` the
/// virtual spaces into a tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Point {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) index: usize,
    pub(crate) vs: usize,
}

/// The content type a linked token holds, tokenized later by subtokenizing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContentType {
    Flow,
    Content,
    String,
    Text,
}

/// Links a token to the previous and next chunk of the same content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) previous: Option<usize>,
    pub(crate) next: Option<usize>,
    pub(crate) content: ContentType,
}

/// Something semantic happening somewhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(crate) kind: Kind,
    pub(crate) name: Name,
    pub(crate) point: Point,
    pub(crate) link: Option<Link>,
}
