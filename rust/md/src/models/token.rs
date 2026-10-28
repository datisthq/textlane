/// A token's name; extensions add their own, so it is an open set rather than an enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Name(pub(crate) &'static str);

impl Name {
    pub(crate) const BLANK_LINE_ENDING: Name = Name("blankLineEnding");
    pub(crate) const BLOCK_QUOTE_MARKER: Name = Name("blockQuoteMarker");
    pub(crate) const BLOCK_QUOTE_PREFIX: Name = Name("blockQuotePrefix");
    pub(crate) const BYTE_ORDER_MARK: Name = Name("byteOrderMark");
    pub(crate) const CONTENT: Name = Name("content");
    pub(crate) const DATA: Name = Name("data");
    pub(crate) const HARD_BREAK_TRAILING: Name = Name("hardBreakTrailing");
    pub(crate) const HEADING_ATX: Name = Name("headingAtx");
    pub(crate) const HEADING_ATX_SEQUENCE: Name = Name("headingAtxSequence");
    pub(crate) const HEADING_ATX_TEXT: Name = Name("headingAtxText");
    pub(crate) const HEADING_SETEXT_UNDERLINE: Name = Name("headingSetextUnderline");
    pub(crate) const LINE_ENDING: Name = Name("lineEnding");
    pub(crate) const PARAGRAPH: Name = Name("paragraph");
    pub(crate) const SPACE_OR_TAB: Name = Name("spaceOrTab");
    pub(crate) const THEMATIC_BREAK: Name = Name("thematicBreak");
    pub(crate) const THEMATIC_BREAK_SEQUENCE: Name = Name("thematicBreakSequence");
}
