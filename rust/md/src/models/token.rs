/// A token's name, extensions' own too: an index into `NAMES`, so comparing two, which every
/// pass over the events does many times, compares two integers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Name(u16);

impl Default for Name {
    fn default() -> Self {
        Name::DATA
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.text())
    }
}

impl Name {
    /// The name `text`, which `NAMES` must list: a constant naming another fails to compile.
    pub(crate) const fn new(text: &str) -> Name {
        let mut index = 0;
        while index < NAMES.len() {
            if is_same(NAMES[index].as_bytes(), text.as_bytes()) {
                return Name(index as u16);
            }
            index += 1;
        }
        panic!("a token name missing from `NAMES`")
    }

    /// The name as micromark spells it.
    pub(crate) fn text(self) -> &'static str {
        NAMES[usize::from(self.0)]
    }
}

const fn is_same(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

// Every token name, those extensions add too
const NAMES: [&str; 273] = [
    "attentionSequence",
    "autolink",
    "autolinkEmail",
    "autolinkMarker",
    "autolinkProtocol",
    "blankLineEnding",
    "blockQuote",
    "blockQuoteMarker",
    "blockQuotePrefix",
    "characterEscape",
    "characterEscapeMarker",
    "characterEscapeValue",
    "characterReference",
    "characterReferenceMarker",
    "characterReferenceMarkerHexadecimal",
    "characterReferenceMarkerNumeric",
    "characterReferenceMarkerSemi",
    "characterReferenceValue",
    "codeFenced",
    "codeFencedFence",
    "codeFencedFenceInfo",
    "codeFencedFenceMeta",
    "codeFencedFenceSequence",
    "codeFlowChunk",
    "codeIndented",
    "codeText",
    "codeTextData",
    "codeTextSequence",
    "content",
    "data",
    "defList",
    "defListDescription",
    "defListDescriptionMarker",
    "defListDescriptionPrefix",
    "defListDescriptionSpread",
    "defListTerm",
    "definition",
    "definitionDestination",
    "definitionDestinationLiteral",
    "definitionDestinationLiteralMarker",
    "definitionDestinationRaw",
    "definitionDestinationString",
    "definitionLabel",
    "definitionLabelMarker",
    "definitionLabelString",
    "definitionMarker",
    "definitionTitle",
    "definitionTitleMarker",
    "definitionTitleString",
    "emphasis",
    "emphasisSequence",
    "emphasisText",
    "frontmatter",
    "frontmatterFence",
    "frontmatterFenceSequence",
    "frontmatterValue",
    "gfmFootnoteCall",
    "gfmFootnoteCallLabelMarker",
    "gfmFootnoteCallMarker",
    "gfmFootnoteCallString",
    "gfmFootnoteDefinition",
    "gfmFootnoteDefinitionLabel",
    "gfmFootnoteDefinitionLabelMarker",
    "gfmFootnoteDefinitionLabelString",
    "gfmFootnoteDefinitionMarker",
    "gfmTable",
    "gfmTableBody",
    "gfmTableCell",
    "gfmTableCellDivider",
    "gfmTableCellText",
    "gfmTableDelimiterCell",
    "gfmTableDelimiterCellValue",
    "gfmTableDelimiterFiller",
    "gfmTableDelimiterMarker",
    "gfmTableDelimiterRow",
    "gfmTableHead",
    "gfmTableRow",
    "hardBreakEscape",
    "hardBreakTrailing",
    "headingAtx",
    "headingAtxSequence",
    "headingAtxText",
    "headingSetext",
    "headingSetextText",
    "headingSetextUnderline",
    "headingSetextUnderlineSequence",
    "htmlFlow",
    "htmlFlowData",
    "htmlText",
    "htmlTextData",
    "image",
    "label",
    "labelEnd",
    "labelImage",
    "labelImageMarker",
    "labelLink",
    "labelMarker",
    "labelText",
    "lineEnding",
    "link",
    "listItem",
    "listItemMarker",
    "listItemPrefix",
    "listItemValue",
    "listOrdered",
    "listUnordered",
    "literalAutolink",
    "literalAutolinkEmail",
    "literalAutolinkHttp",
    "literalAutolinkWww",
    "mathFlow",
    "mathFlowFence",
    "mathFlowFenceMeta",
    "mathFlowFenceSequence",
    "mathFlowValue",
    "mathText",
    "mathTextData",
    "mathTextSequence",
    "paragraph",
    "reference",
    "referenceMarker",
    "referenceString",
    "resource",
    "resourceDestination",
    "resourceDestinationLiteral",
    "resourceDestinationLiteralMarker",
    "resourceDestinationRaw",
    "resourceDestinationString",
    "resourceMarker",
    "resourceTitle",
    "resourceTitleMarker",
    "resourceTitleString",
    "spaceOrTab",
    "strikethrough",
    "strikethroughSequence",
    "strikethroughSequenceTemporary",
    "strikethroughText",
    "strong",
    "strongSequence",
    "strongText",
    "taskListCheck",
    "taskListCheckMarker",
    "taskListCheckValueChecked",
    "taskListCheckValueUnchecked",
    "thematicBreak",
    "thematicBreakSequence",
    "mdxJsxFlowTag",
    "mdxJsxFlowTagMarker",
    "mdxJsxFlowTagClosingMarker",
    "mdxJsxFlowTagSelfClosingMarker",
    "mdxJsxFlowTagName",
    "mdxJsxFlowTagNamePrimary",
    "mdxJsxFlowTagNameMemberMarker",
    "mdxJsxFlowTagNameMember",
    "mdxJsxFlowTagNamePrefixMarker",
    "mdxJsxFlowTagNameLocal",
    "mdxJsxFlowTagExpressionAttribute",
    "mdxJsxFlowTagExpressionAttributeMarker",
    "mdxJsxFlowTagExpressionAttributeValue",
    "mdxJsxFlowTagAttribute",
    "mdxJsxFlowTagAttributeName",
    "mdxJsxFlowTagAttributeNamePrimary",
    "mdxJsxFlowTagAttributeNamePrefixMarker",
    "mdxJsxFlowTagAttributeNameLocal",
    "mdxJsxFlowTagAttributeInitializerMarker",
    "mdxJsxFlowTagAttributeValueLiteral",
    "mdxJsxFlowTagAttributeValueLiteralMarker",
    "mdxJsxFlowTagAttributeValueLiteralValue",
    "mdxJsxFlowTagAttributeValueExpression",
    "mdxJsxFlowTagAttributeValueExpressionMarker",
    "mdxJsxFlowTagAttributeValueExpressionValue",
    "mdxJsxTextTag",
    "mdxJsxTextTagMarker",
    "mdxJsxTextTagClosingMarker",
    "mdxJsxTextTagSelfClosingMarker",
    "mdxJsxTextTagName",
    "mdxJsxTextTagNamePrimary",
    "mdxJsxTextTagNameMemberMarker",
    "mdxJsxTextTagNameMember",
    "mdxJsxTextTagNamePrefixMarker",
    "mdxJsxTextTagNameLocal",
    "mdxJsxTextTagExpressionAttribute",
    "mdxJsxTextTagExpressionAttributeMarker",
    "mdxJsxTextTagExpressionAttributeValue",
    "mdxJsxTextTagAttribute",
    "mdxJsxTextTagAttributeName",
    "mdxJsxTextTagAttributeNamePrimary",
    "mdxJsxTextTagAttributeNamePrefixMarker",
    "mdxJsxTextTagAttributeNameLocal",
    "mdxJsxTextTagAttributeInitializerMarker",
    "mdxJsxTextTagAttributeValueLiteral",
    "mdxJsxTextTagAttributeValueLiteralMarker",
    "mdxJsxTextTagAttributeValueLiteralValue",
    "mdxJsxTextTagAttributeValueExpression",
    "mdxJsxTextTagAttributeValueExpressionMarker",
    "mdxJsxTextTagAttributeValueExpressionValue",
    "mdxFlowExpression",
    "mdxFlowExpressionMarker",
    "mdxFlowExpressionChunk",
    "mdxTextExpression",
    "mdxTextExpressionMarker",
    "mdxTextExpressionChunk",
    "mdxjsEsm",
    "mdxjsEsmData",
    "esWhitespace",
    "linePrefix",
    "directiveTextName",
    "directiveTextLabel",
    "directiveTextLabelMarker",
    "directiveTextLabelString",
    "directiveTextAttributes",
    "directiveTextAttributesMarker",
    "directiveTextAttribute",
    "directiveTextAttributeId",
    "directiveTextAttributeIdMarker",
    "directiveTextAttributeIdValue",
    "directiveTextAttributeClass",
    "directiveTextAttributeClassMarker",
    "directiveTextAttributeClassValue",
    "directiveTextAttributeName",
    "directiveTextAttributeInitializerMarker",
    "directiveTextAttributeValueLiteral",
    "directiveTextAttributeValue",
    "directiveTextAttributeValueMarker",
    "directiveTextAttributeValueData",
    "directiveLeafName",
    "directiveLeafLabel",
    "directiveLeafLabelMarker",
    "directiveLeafLabelString",
    "directiveLeafAttributes",
    "directiveLeafAttributesMarker",
    "directiveLeafAttribute",
    "directiveLeafAttributeId",
    "directiveLeafAttributeIdMarker",
    "directiveLeafAttributeIdValue",
    "directiveLeafAttributeClass",
    "directiveLeafAttributeClassMarker",
    "directiveLeafAttributeClassValue",
    "directiveLeafAttributeName",
    "directiveLeafAttributeInitializerMarker",
    "directiveLeafAttributeValueLiteral",
    "directiveLeafAttributeValue",
    "directiveLeafAttributeValueMarker",
    "directiveLeafAttributeValueData",
    "directiveContainerName",
    "directiveContainerLabel",
    "directiveContainerLabelMarker",
    "directiveContainerLabelString",
    "directiveContainerAttributes",
    "directiveContainerAttributesMarker",
    "directiveContainerAttribute",
    "directiveContainerAttributeId",
    "directiveContainerAttributeIdMarker",
    "directiveContainerAttributeIdValue",
    "directiveContainerAttributeClass",
    "directiveContainerAttributeClassMarker",
    "directiveContainerAttributeClassValue",
    "directiveContainerAttributeName",
    "directiveContainerAttributeInitializerMarker",
    "directiveContainerAttributeValueLiteral",
    "directiveContainerAttributeValue",
    "directiveContainerAttributeValueMarker",
    "directiveContainerAttributeValueData",
    "directiveText",
    "directiveTextMarker",
    "directiveLeaf",
    "directiveLeafSequence",
    "directiveContainer",
    "directiveContainerFence",
    "directiveContainerSequence",
    "directiveContainerContent",
    "chunkText",
    "chunkDocument",
];

impl Name {
    pub(crate) const ATTENTION_SEQUENCE: Name = Name::new("attentionSequence");
    pub(crate) const AUTOLINK: Name = Name::new("autolink");
    pub(crate) const AUTOLINK_EMAIL: Name = Name::new("autolinkEmail");
    pub(crate) const AUTOLINK_MARKER: Name = Name::new("autolinkMarker");
    pub(crate) const AUTOLINK_PROTOCOL: Name = Name::new("autolinkProtocol");
    pub(crate) const BLANK_LINE_ENDING: Name = Name::new("blankLineEnding");
    pub(crate) const BLOCK_QUOTE: Name = Name::new("blockQuote");
    pub(crate) const BLOCK_QUOTE_MARKER: Name = Name::new("blockQuoteMarker");
    pub(crate) const BLOCK_QUOTE_PREFIX: Name = Name::new("blockQuotePrefix");
    pub(crate) const CHARACTER_ESCAPE: Name = Name::new("characterEscape");
    pub(crate) const CHARACTER_ESCAPE_MARKER: Name = Name::new("characterEscapeMarker");
    pub(crate) const CHARACTER_ESCAPE_VALUE: Name = Name::new("characterEscapeValue");
    pub(crate) const CHARACTER_REFERENCE: Name = Name::new("characterReference");
    pub(crate) const CHARACTER_REFERENCE_MARKER: Name =
        Name::new("characterReferenceMarker");
    pub(crate) const CHARACTER_REFERENCE_MARKER_HEXADECIMAL: Name =
        Name::new("characterReferenceMarkerHexadecimal");
    pub(crate) const CHARACTER_REFERENCE_MARKER_NUMERIC: Name =
        Name::new("characterReferenceMarkerNumeric");
    pub(crate) const CHARACTER_REFERENCE_MARKER_SEMI: Name =
        Name::new("characterReferenceMarkerSemi");
    pub(crate) const CHARACTER_REFERENCE_VALUE: Name =
        Name::new("characterReferenceValue");
    pub(crate) const CODE_FENCED: Name = Name::new("codeFenced");
    pub(crate) const CODE_FENCED_FENCE: Name = Name::new("codeFencedFence");
    pub(crate) const CODE_FENCED_FENCE_INFO: Name = Name::new("codeFencedFenceInfo");
    pub(crate) const CODE_FENCED_FENCE_META: Name = Name::new("codeFencedFenceMeta");
    pub(crate) const CODE_FENCED_FENCE_SEQUENCE: Name =
        Name::new("codeFencedFenceSequence");
    pub(crate) const CODE_FLOW_CHUNK: Name = Name::new("codeFlowChunk");
    pub(crate) const CODE_INDENTED: Name = Name::new("codeIndented");
    pub(crate) const CODE_TEXT: Name = Name::new("codeText");
    pub(crate) const CODE_TEXT_DATA: Name = Name::new("codeTextData");
    pub(crate) const CODE_TEXT_SEQUENCE: Name = Name::new("codeTextSequence");
    pub(crate) const CONTENT: Name = Name::new("content");
    pub(crate) const DATA: Name = Name::new("data");
    pub(crate) const DEF_LIST: Name = Name::new("defList");
    pub(crate) const DEF_LIST_DESCRIPTION: Name = Name::new("defListDescription");
    pub(crate) const DEF_LIST_DESCRIPTION_MARKER: Name =
        Name::new("defListDescriptionMarker");
    pub(crate) const DEF_LIST_DESCRIPTION_PREFIX: Name =
        Name::new("defListDescriptionPrefix");
    pub(crate) const DEF_LIST_DESCRIPTION_SPREAD: Name =
        Name::new("defListDescriptionSpread");
    pub(crate) const DEF_LIST_TERM: Name = Name::new("defListTerm");
    pub(crate) const DEFINITION: Name = Name::new("definition");
    pub(crate) const DEFINITION_DESTINATION: Name = Name::new("definitionDestination");
    pub(crate) const DEFINITION_DESTINATION_LITERAL: Name =
        Name::new("definitionDestinationLiteral");
    pub(crate) const DEFINITION_DESTINATION_LITERAL_MARKER: Name =
        Name::new("definitionDestinationLiteralMarker");
    pub(crate) const DEFINITION_DESTINATION_RAW: Name =
        Name::new("definitionDestinationRaw");
    pub(crate) const DEFINITION_DESTINATION_STRING: Name =
        Name::new("definitionDestinationString");
    pub(crate) const DEFINITION_LABEL: Name = Name::new("definitionLabel");
    pub(crate) const DEFINITION_LABEL_MARKER: Name = Name::new("definitionLabelMarker");
    pub(crate) const DEFINITION_LABEL_STRING: Name = Name::new("definitionLabelString");
    pub(crate) const DEFINITION_MARKER: Name = Name::new("definitionMarker");
    pub(crate) const DEFINITION_TITLE: Name = Name::new("definitionTitle");
    pub(crate) const DEFINITION_TITLE_MARKER: Name = Name::new("definitionTitleMarker");
    pub(crate) const DEFINITION_TITLE_STRING: Name = Name::new("definitionTitleString");
    pub(crate) const EMPHASIS: Name = Name::new("emphasis");
    pub(crate) const EMPHASIS_SEQUENCE: Name = Name::new("emphasisSequence");
    pub(crate) const EMPHASIS_TEXT: Name = Name::new("emphasisText");
    pub(crate) const FRONTMATTER: Name = Name::new("frontmatter");
    pub(crate) const FRONTMATTER_FENCE: Name = Name::new("frontmatterFence");
    pub(crate) const FRONTMATTER_SEQUENCE: Name = Name::new("frontmatterFenceSequence");
    pub(crate) const FRONTMATTER_VALUE: Name = Name::new("frontmatterValue");
    pub(crate) const GFM_FOOTNOTE_CALL: Name = Name::new("gfmFootnoteCall");
    pub(crate) const GFM_FOOTNOTE_CALL_LABEL_MARKER: Name =
        Name::new("gfmFootnoteCallLabelMarker");
    pub(crate) const GFM_FOOTNOTE_CALL_MARKER: Name = Name::new("gfmFootnoteCallMarker");
    pub(crate) const GFM_FOOTNOTE_CALL_STRING: Name = Name::new("gfmFootnoteCallString");
    pub(crate) const GFM_FOOTNOTE_DEFINITION: Name = Name::new("gfmFootnoteDefinition");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_LABEL: Name =
        Name::new("gfmFootnoteDefinitionLabel");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_LABEL_MARKER: Name =
        Name::new("gfmFootnoteDefinitionLabelMarker");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_LABEL_STRING: Name =
        Name::new("gfmFootnoteDefinitionLabelString");
    pub(crate) const GFM_FOOTNOTE_DEFINITION_MARKER: Name =
        Name::new("gfmFootnoteDefinitionMarker");
    pub(crate) const GFM_TABLE: Name = Name::new("gfmTable");
    pub(crate) const GFM_TABLE_BODY: Name = Name::new("gfmTableBody");
    pub(crate) const GFM_TABLE_CELL: Name = Name::new("gfmTableCell");
    pub(crate) const GFM_TABLE_CELL_DIVIDER: Name = Name::new("gfmTableCellDivider");
    pub(crate) const GFM_TABLE_CELL_TEXT: Name = Name::new("gfmTableCellText");
    pub(crate) const GFM_TABLE_DELIMITER_CELL: Name = Name::new("gfmTableDelimiterCell");
    pub(crate) const GFM_TABLE_DELIMITER_CELL_VALUE: Name =
        Name::new("gfmTableDelimiterCellValue");
    pub(crate) const GFM_TABLE_DELIMITER_FILLER: Name =
        Name::new("gfmTableDelimiterFiller");
    pub(crate) const GFM_TABLE_DELIMITER_MARKER: Name =
        Name::new("gfmTableDelimiterMarker");
    pub(crate) const GFM_TABLE_DELIMITER_ROW: Name = Name::new("gfmTableDelimiterRow");
    pub(crate) const GFM_TABLE_HEAD: Name = Name::new("gfmTableHead");
    pub(crate) const GFM_TABLE_ROW: Name = Name::new("gfmTableRow");
    pub(crate) const HARD_BREAK_ESCAPE: Name = Name::new("hardBreakEscape");
    pub(crate) const HARD_BREAK_TRAILING: Name = Name::new("hardBreakTrailing");
    pub(crate) const HEADING_ATX: Name = Name::new("headingAtx");
    pub(crate) const HEADING_ATX_SEQUENCE: Name = Name::new("headingAtxSequence");
    pub(crate) const HEADING_ATX_TEXT: Name = Name::new("headingAtxText");
    pub(crate) const HEADING_SETEXT: Name = Name::new("headingSetext");
    pub(crate) const HEADING_SETEXT_TEXT: Name = Name::new("headingSetextText");
    pub(crate) const HEADING_SETEXT_UNDERLINE: Name = Name::new("headingSetextUnderline");
    pub(crate) const HEADING_SETEXT_UNDERLINE_SEQUENCE: Name =
        Name::new("headingSetextUnderlineSequence");
    pub(crate) const HTML_FLOW: Name = Name::new("htmlFlow");
    pub(crate) const HTML_FLOW_DATA: Name = Name::new("htmlFlowData");
    pub(crate) const HTML_TEXT: Name = Name::new("htmlText");
    pub(crate) const HTML_TEXT_DATA: Name = Name::new("htmlTextData");
    pub(crate) const IMAGE: Name = Name::new("image");
    pub(crate) const LABEL: Name = Name::new("label");
    pub(crate) const LABEL_END: Name = Name::new("labelEnd");
    pub(crate) const LABEL_IMAGE: Name = Name::new("labelImage");
    pub(crate) const LABEL_IMAGE_MARKER: Name = Name::new("labelImageMarker");
    pub(crate) const LABEL_LINK: Name = Name::new("labelLink");
    pub(crate) const LABEL_MARKER: Name = Name::new("labelMarker");
    pub(crate) const LABEL_TEXT: Name = Name::new("labelText");
    pub(crate) const LINE_ENDING: Name = Name::new("lineEnding");
    pub(crate) const LINK: Name = Name::new("link");
    pub(crate) const LIST_ITEM: Name = Name::new("listItem");
    pub(crate) const LIST_ITEM_MARKER: Name = Name::new("listItemMarker");
    pub(crate) const LIST_ITEM_PREFIX: Name = Name::new("listItemPrefix");
    pub(crate) const LIST_ITEM_VALUE: Name = Name::new("listItemValue");
    pub(crate) const LIST_ORDERED: Name = Name::new("listOrdered");
    pub(crate) const LIST_UNORDERED: Name = Name::new("listUnordered");
    pub(crate) const LITERAL_AUTOLINK: Name = Name::new("literalAutolink");
    pub(crate) const LITERAL_AUTOLINK_EMAIL: Name = Name::new("literalAutolinkEmail");
    pub(crate) const LITERAL_AUTOLINK_HTTP: Name = Name::new("literalAutolinkHttp");
    pub(crate) const LITERAL_AUTOLINK_WWW: Name = Name::new("literalAutolinkWww");
    pub(crate) const MATH_FLOW: Name = Name::new("mathFlow");
    pub(crate) const MATH_FLOW_FENCE: Name = Name::new("mathFlowFence");
    pub(crate) const MATH_FLOW_FENCE_META: Name = Name::new("mathFlowFenceMeta");
    pub(crate) const MATH_FLOW_FENCE_SEQUENCE: Name = Name::new("mathFlowFenceSequence");
    pub(crate) const MATH_FLOW_VALUE: Name = Name::new("mathFlowValue");
    pub(crate) const MATH_TEXT: Name = Name::new("mathText");
    pub(crate) const MATH_TEXT_DATA: Name = Name::new("mathTextData");
    pub(crate) const MATH_TEXT_SEQUENCE: Name = Name::new("mathTextSequence");
    pub(crate) const PARAGRAPH: Name = Name::new("paragraph");
    pub(crate) const REFERENCE: Name = Name::new("reference");
    pub(crate) const REFERENCE_MARKER: Name = Name::new("referenceMarker");
    pub(crate) const REFERENCE_STRING: Name = Name::new("referenceString");
    pub(crate) const RESOURCE: Name = Name::new("resource");
    pub(crate) const RESOURCE_DESTINATION: Name = Name::new("resourceDestination");
    pub(crate) const RESOURCE_DESTINATION_LITERAL: Name =
        Name::new("resourceDestinationLiteral");
    pub(crate) const RESOURCE_DESTINATION_LITERAL_MARKER: Name =
        Name::new("resourceDestinationLiteralMarker");
    pub(crate) const RESOURCE_DESTINATION_RAW: Name = Name::new("resourceDestinationRaw");
    pub(crate) const RESOURCE_DESTINATION_STRING: Name =
        Name::new("resourceDestinationString");
    pub(crate) const RESOURCE_MARKER: Name = Name::new("resourceMarker");
    pub(crate) const RESOURCE_TITLE: Name = Name::new("resourceTitle");
    pub(crate) const RESOURCE_TITLE_MARKER: Name = Name::new("resourceTitleMarker");
    pub(crate) const RESOURCE_TITLE_STRING: Name = Name::new("resourceTitleString");
    pub(crate) const SPACE_OR_TAB: Name = Name::new("spaceOrTab");
    pub(crate) const STRIKETHROUGH: Name = Name::new("strikethrough");
    pub(crate) const STRIKETHROUGH_SEQUENCE: Name = Name::new("strikethroughSequence");
    pub(crate) const STRIKETHROUGH_SEQUENCE_TEMPORARY: Name =
        Name::new("strikethroughSequenceTemporary");
    pub(crate) const STRIKETHROUGH_TEXT: Name = Name::new("strikethroughText");
    pub(crate) const STRONG: Name = Name::new("strong");
    pub(crate) const STRONG_SEQUENCE: Name = Name::new("strongSequence");
    pub(crate) const STRONG_TEXT: Name = Name::new("strongText");
    pub(crate) const TASK_LIST_CHECK: Name = Name::new("taskListCheck");
    pub(crate) const TASK_LIST_CHECK_MARKER: Name = Name::new("taskListCheckMarker");
    pub(crate) const TASK_LIST_CHECK_VALUE_CHECKED: Name =
        Name::new("taskListCheckValueChecked");
    pub(crate) const TASK_LIST_CHECK_VALUE_UNCHECKED: Name =
        Name::new("taskListCheckValueUnchecked");
    pub(crate) const THEMATIC_BREAK: Name = Name::new("thematicBreak");
    pub(crate) const THEMATIC_BREAK_SEQUENCE: Name = Name::new("thematicBreakSequence");
}
