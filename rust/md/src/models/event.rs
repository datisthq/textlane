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
    /// Text whose trailing whitespace stays, as in a directive's label.
    TextTrailing,
    /// A document of its own, as a container directive's content.
    Document,
}

/// Links a token to the previous and next chunk of the same content, by event index.
// u32 keeps an event in one cache line: every pass over the events pays its size
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) previous: Option<u32>,
    pub(crate) next: Option<u32>,
    pub(crate) content: ContentType,
}

impl Link {
    /// Where the previous chunk's enter is.
    pub(crate) fn previous(&self) -> Option<usize> {
        self.previous.map(|index| index as usize)
    }

    /// Where the next chunk's enter is.
    pub(crate) fn next(&self) -> Option<usize> {
        self.next.map(|index| index as usize)
    }

    /// An event index as a link holds it.
    pub(crate) fn at(index: usize) -> Option<u32> {
        Some(index as u32)
    }
}

/// Something semantic happening somewhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(crate) kind: Kind,
    pub(crate) name: Name,
    pub(crate) point: Point,
    pub(crate) link: Option<Link>,
}
