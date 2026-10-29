/// A token's name; extensions add their own, so it is an open set rather than an enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Name(pub(crate) &'static str);

impl Default for Name {
    fn default() -> Self {
        Name::DATA
    }
}

impl Name {
    pub(crate) const ATTENTION_SEQUENCE: Name = Name("attentionSequence");
    pub(crate) const AUTOLINK: Name = Name("autolink");
    pub(crate) const AUTOLINK_EMAIL: Name = Name("autolinkEmail");
    pub(crate) const AUTOLINK_MARKER: Name = Name("autolinkMarker");
    pub(crate) const AUTOLINK_PROTOCOL: Name = Name("autolinkProtocol");
    pub(crate) const BLANK_LINE_ENDING: Name = Name("blankLineEnding");
    pub(crate) const BLOCK_QUOTE: Name = Name("blockQuote");
    pub(crate) const BLOCK_QUOTE_MARKER: Name = Name("blockQuoteMarker");
    pub(crate) const BLOCK_QUOTE_PREFIX: Name = Name("blockQuotePrefix");
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
    pub(crate) const DEF_LIST: Name = Name("defList");
    pub(crate) const DEF_LIST_DESCRIPTION: Name = Name("defListDescription");
    pub(crate) const DEF_LIST_DESCRIPTION_MARKER: Name = Name("defListDescriptionMarker");
    pub(crate) const DEF_LIST_DESCRIPTION_PREFIX: Name = Name("defListDescriptionPrefix");
    pub(crate) const DEF_LIST_DESCRIPTION_SPREAD: Name = Name("defListDescriptionSpread");
    pub(crate) const DEF_LIST_TERM: Name = Name("defListTerm");
    pub(crate) const DEFINITION: Name = Name("definition");
    pub(crate) const DEFINITION_DESTINATION: Name = Name("definitionDestination");
    pub(crate) const DEFINITION_DESTINATION_LITERAL: Name =
        Name("definitionDestinationLiteral");
    pub(crate) const DEFINITION_DESTINATION_LITERAL_MARKER: Name =
        Name("definitionDestinationLiteralMarker");
    pub(crate) const DEFINITION_DESTINATION_RAW: Name = Name("definitionDestinationRaw");
    pub(crate) const DEFINITION_DESTINATION_STRING: Name =
        Name("definitionDestinationString");
    pub(crate) const DEFINITION_LABEL: Name = Name("definitionLabel");
    pub(crate) const DEFINITION_LABEL_MARKER: Name = Name("definitionLabelMarker");
    pub(crate) const DEFINITION_LABEL_STRING: Name = Name("definitionLabelString");
    pub(crate) const DEFINITION_MARKER: Name = Name("definitionMarker");
    pub(crate) const DEFINITION_TITLE: Name = Name("definitionTitle");
    pub(crate) const DEFINITION_TITLE_MARKER: Name = Name("definitionTitleMarker");
    pub(crate) const DEFINITION_TITLE_STRING: Name = Name("definitionTitleString");
    pub(crate) const EMPHASIS: Name = Name("emphasis");
    pub(crate) const EMPHASIS_SEQUENCE: Name = Name("emphasisSequence");
    pub(crate) const EMPHASIS_TEXT: Name = Name("emphasisText");
    pub(crate) const FRONTMATTER: Name = Name("frontmatter");
    pub(crate) const FRONTMATTER_FENCE: Name = Name("frontmatterFence");
    pub(crate) const FRONTMATTER_SEQUENCE: Name = Name("frontmatterFenceSequence");
    pub(crate) const FRONTMATTER_VALUE: Name = Name("frontmatterValue");
    pub(crate) const GFM_FOOTNOTE_CALL: Name = Name("gfmFootnoteCall");
    pub(crate) const GFM_FOOTNOTE_CALL_LABEL_MARKER: Name =
        Name("gfmFootnoteCallLabelMarker");
    pub(crate) const GFM_FOOTNOTE_CALL_MARKER: Name = Name("gfmFootnoteCallMarker");
    pub(crate) const GFM_FOOTNOTE_CALL_STRING: Name = Name("gfmFootnoteCallString");
    pub(crate) const GFM_FOOTNOTE_DEFINITION: Name = Name("gfmFootnoteDefinition");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_LABEL: Name =
        Name("gfmFootnoteDefinitionLabel");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_LABEL_MARKER: Name =
        Name("gfmFootnoteDefinitionLabelMarker");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_LABEL_STRING: Name =
        Name("gfmFootnoteDefinitionLabelString");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_MARKER: Name =
        Name("gfmFootnoteDefinitionMarker");
    pub(crate) const GFM_TABLE: Name = Name("gfmTable");
    pub(crate) const GFM_TABLE_BODY: Name = Name("gfmTableBody");
    pub(crate) const GFM_TABLE_CELL: Name = Name("gfmTableCell");
    pub(crate) const GFM_TABLE_CELL_DIVIDER: Name = Name("gfmTableCellDivider");
    pub(crate) const GFM_TABLE_CELL_TEXT: Name = Name("gfmTableCellText");
    pub(crate) const GFM_TABLE_DELIMITER_CELL: Name = Name("gfmTableDelimiterCell");
    pub(crate) const GFM_TABLE_DELIMITER_CELL_VALUE: Name =
        Name("gfmTableDelimiterCellValue");
    pub(crate) const GFM_TABLE_DELIMITER_FILLER: Name = Name("gfmTableDelimiterFiller");
    pub(crate) const GFM_TABLE_DELIMITER_MARKER: Name = Name("gfmTableDelimiterMarker");
    pub(crate) const GFM_TABLE_DELIMITER_ROW: Name = Name("gfmTableDelimiterRow");
    pub(crate) const GFM_TABLE_HEAD: Name = Name("gfmTableHead");
    pub(crate) const GFM_TABLE_ROW: Name = Name("gfmTableRow");
    pub(crate) const HARD_BREAK_ESCAPE: Name = Name("hardBreakEscape");
    pub(crate) const HARD_BREAK_TRAILING: Name = Name("hardBreakTrailing");
    pub(crate) const HEADING_ATX: Name = Name("headingAtx");
    pub(crate) const HEADING_ATX_SEQUENCE: Name = Name("headingAtxSequence");
    pub(crate) const HEADING_ATX_TEXT: Name = Name("headingAtxText");
    pub(crate) const HEADING_SETEXT: Name = Name("headingSetext");
    pub(crate) const HEADING_SETEXT_TEXT: Name = Name("headingSetextText");
    pub(crate) const HEADING_SETEXT_UNDERLINE: Name = Name("headingSetextUnderline");
    pub(crate) const HEADING_SETEXT_UNDERLINE_SEQUENCE: Name =
        Name("headingSetextUnderlineSequence");
    pub(crate) const HTML_FLOW: Name = Name("htmlFlow");
    pub(crate) const HTML_FLOW_DATA: Name = Name("htmlFlowData");
    pub(crate) const HTML_TEXT: Name = Name("htmlText");
    pub(crate) const HTML_TEXT_DATA: Name = Name("htmlTextData");
    pub(crate) const IMAGE: Name = Name("image");
    pub(crate) const LABEL: Name = Name("label");
    pub(crate) const LABEL_END: Name = Name("labelEnd");
    pub(crate) const LABEL_IMAGE: Name = Name("labelImage");
    pub(crate) const LABEL_IMAGE_MARKER: Name = Name("labelImageMarker");
    pub(crate) const LABEL_LINK: Name = Name("labelLink");
    pub(crate) const LABEL_MARKER: Name = Name("labelMarker");
    pub(crate) const LABEL_TEXT: Name = Name("labelText");
    pub(crate) const LINE_ENDING: Name = Name("lineEnding");
    pub(crate) const LINK: Name = Name("link");
    pub(crate) const LIST_ITEM: Name = Name("listItem");
    pub(crate) const LIST_ITEM_MARKER: Name = Name("listItemMarker");
    pub(crate) const LIST_ITEM_PREFIX: Name = Name("listItemPrefix");
    pub(crate) const LIST_ITEM_VALUE: Name = Name("listItemValue");
    pub(crate) const LIST_ORDERED: Name = Name("listOrdered");
    pub(crate) const LIST_UNORDERED: Name = Name("listUnordered");
    pub(crate) const LITERAL_AUTOLINK: Name = Name("literalAutolink");
    pub(crate) const LITERAL_AUTOLINK_EMAIL: Name = Name("literalAutolinkEmail");
    pub(crate) const LITERAL_AUTOLINK_HTTP: Name = Name("literalAutolinkHttp");
    pub(crate) const LITERAL_AUTOLINK_WWW: Name = Name("literalAutolinkWww");
    pub(crate) const MATH_FLOW: Name = Name("mathFlow");
    pub(crate) const MATH_FLOW_FENCE: Name = Name("mathFlowFence");
    pub(crate) const MATH_FLOW_FENCE_META: Name = Name("mathFlowFenceMeta");
    pub(crate) const MATH_FLOW_FENCE_SEQUENCE: Name = Name("mathFlowFenceSequence");
    pub(crate) const MATH_FLOW_VALUE: Name = Name("mathFlowValue");
    pub(crate) const MATH_TEXT: Name = Name("mathText");
    pub(crate) const MATH_TEXT_DATA: Name = Name("mathTextData");
    pub(crate) const MATH_TEXT_SEQUENCE: Name = Name("mathTextSequence");
    pub(crate) const PARAGRAPH: Name = Name("paragraph");
    pub(crate) const REFERENCE: Name = Name("reference");
    pub(crate) const REFERENCE_MARKER: Name = Name("referenceMarker");
    pub(crate) const REFERENCE_STRING: Name = Name("referenceString");
    pub(crate) const RESOURCE: Name = Name("resource");
    pub(crate) const RESOURCE_DESTINATION: Name = Name("resourceDestination");
    pub(crate) const RESOURCE_DESTINATION_LITERAL: Name =
        Name("resourceDestinationLiteral");
    pub(crate) const RESOURCE_DESTINATION_LITERAL_MARKER: Name =
        Name("resourceDestinationLiteralMarker");
    pub(crate) const RESOURCE_DESTINATION_RAW: Name = Name("resourceDestinationRaw");
    pub(crate) const RESOURCE_DESTINATION_STRING: Name =
        Name("resourceDestinationString");
    pub(crate) const RESOURCE_MARKER: Name = Name("resourceMarker");
    pub(crate) const RESOURCE_TITLE: Name = Name("resourceTitle");
    pub(crate) const RESOURCE_TITLE_MARKER: Name = Name("resourceTitleMarker");
    pub(crate) const RESOURCE_TITLE_STRING: Name = Name("resourceTitleString");
    pub(crate) const SPACE_OR_TAB: Name = Name("spaceOrTab");
    pub(crate) const STRIKETHROUGH: Name = Name("strikethrough");
    pub(crate) const STRIKETHROUGH_SEQUENCE: Name = Name("strikethroughSequence");
    pub(crate) const STRIKETHROUGH_SEQUENCE_TEMPORARY: Name =
        Name("strikethroughSequenceTemporary");
    pub(crate) const STRIKETHROUGH_TEXT: Name = Name("strikethroughText");
    pub(crate) const STRONG: Name = Name("strong");
    pub(crate) const STRONG_SEQUENCE: Name = Name("strongSequence");
    pub(crate) const STRONG_TEXT: Name = Name("strongText");
    pub(crate) const TASK_LIST_CHECK: Name = Name("taskListCheck");
    pub(crate) const TASK_LIST_CHECK_MARKER: Name = Name("taskListCheckMarker");
    pub(crate) const TASK_LIST_CHECK_VALUE_CHECKED: Name =
        Name("taskListCheckValueChecked");
    pub(crate) const TASK_LIST_CHECK_VALUE_UNCHECKED: Name =
        Name("taskListCheckValueUnchecked");
    pub(crate) const THEMATIC_BREAK: Name = Name("thematicBreak");
    pub(crate) const THEMATIC_BREAK_SEQUENCE: Name = Name("thematicBreakSequence");
}
