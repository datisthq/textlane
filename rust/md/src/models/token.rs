/// A token's name; extensions add their own, so it is an open set rather than an enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Name(pub(crate) &'static str);

impl Name {
    pub(crate) const ATTENTION_SEQUENCE: Name = Name("attentionSequence");
    pub(crate) const BLANK_LINE_ENDING: Name = Name("blankLineEnding");
    pub(crate) const BLOCK_QUOTE: Name = Name("blockQuote");
    pub(crate) const BLOCK_QUOTE_MARKER: Name = Name("blockQuoteMarker");
    pub(crate) const BLOCK_QUOTE_PREFIX: Name = Name("blockQuotePrefix");
    pub(crate) const BYTE_ORDER_MARK: Name = Name("byteOrderMark");
    pub(crate) const CHARACTER_ESCAPE: Name = Name("characterEscape");
    pub(crate) const CHARACTER_ESCAPE_MARKER: Name = Name("characterEscapeMarker");
    pub(crate) const CHARACTER_ESCAPE_VALUE: Name = Name("characterEscapeValue");
    pub(crate) const CHARACTER_REFERENCE: Name = Name("characterReference");
    pub(crate) const CHARACTER_REFERENCE_MARKER: Name = Name("characterReferenceMarker");
    pub(crate) const CHARACTER_REFERENCE_MARKER_HEXADECIMAL: Name =
        Name("characterReferenceMarkerHexadecimal");
    pub(crate) const CHARACTER_REFERENCE_MARKER_NUMERIC: Name =
        Name("characterReferenceMarkerNumeric");
    pub(crate) const CHARACTER_REFERENCE_MARKER_SEMI: Name =
        Name("characterReferenceMarkerSemi");
    pub(crate) const CHARACTER_REFERENCE_VALUE: Name = Name("characterReferenceValue");
    pub(crate) const CODE_FENCED: Name = Name("codeFenced");
    pub(crate) const CODE_FENCED_FENCE: Name = Name("codeFencedFence");
    pub(crate) const CODE_FENCED_FENCE_INFO: Name = Name("codeFencedFenceInfo");
    pub(crate) const CODE_FENCED_FENCE_META: Name = Name("codeFencedFenceMeta");
    pub(crate) const CODE_FENCED_FENCE_SEQUENCE: Name = Name("codeFencedFenceSequence");
    pub(crate) const CODE_FLOW_CHUNK: Name = Name("codeFlowChunk");
    pub(crate) const CODE_INDENTED: Name = Name("codeIndented");
    pub(crate) const CODE_TEXT: Name = Name("codeText");
    pub(crate) const CODE_TEXT_DATA: Name = Name("codeTextData");
    pub(crate) const CODE_TEXT_SEQUENCE: Name = Name("codeTextSequence");
    pub(crate) const CONTENT: Name = Name("content");
    pub(crate) const DATA: Name = Name("data");
    pub(crate) const EMPHASIS: Name = Name("emphasis");
    pub(crate) const EMPHASIS_SEQUENCE: Name = Name("emphasisSequence");
    pub(crate) const EMPHASIS_TEXT: Name = Name("emphasisText");
    pub(crate) const HARD_BREAK_ESCAPE: Name = Name("hardBreakEscape");
    pub(crate) const HARD_BREAK_TRAILING: Name = Name("hardBreakTrailing");
    pub(crate) const HEADING_ATX: Name = Name("headingAtx");
    pub(crate) const HEADING_ATX_SEQUENCE: Name = Name("headingAtxSequence");
    pub(crate) const HEADING_ATX_TEXT: Name = Name("headingAtxText");
    pub(crate) const HEADING_SETEXT_UNDERLINE: Name = Name("headingSetextUnderline");
    pub(crate) const LINE_ENDING: Name = Name("lineEnding");
    pub(crate) const LIST_ITEM: Name = Name("listItem");
    pub(crate) const LIST_ITEM_MARKER: Name = Name("listItemMarker");
    pub(crate) const LIST_ITEM_PREFIX: Name = Name("listItemPrefix");
    pub(crate) const LIST_ITEM_VALUE: Name = Name("listItemValue");
    pub(crate) const LIST_ORDERED: Name = Name("listOrdered");
    pub(crate) const LIST_UNORDERED: Name = Name("listUnordered");
    pub(crate) const PARAGRAPH: Name = Name("paragraph");
    pub(crate) const SPACE_OR_TAB: Name = Name("spaceOrTab");
    pub(crate) const STRONG: Name = Name("strong");
    pub(crate) const STRONG_SEQUENCE: Name = Name("strongSequence");
    pub(crate) const STRONG_TEXT: Name = Name("strongText");
    pub(crate) const THEMATIC_BREAK: Name = Name("thematicBreak");
    pub(crate) const THEMATIC_BREAK_SEQUENCE: Name = Name("thematicBreakSequence");
}
