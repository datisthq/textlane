/// hast-util-from-html's reason for each parse5 error code.
pub(crate) fn find_reason(code: &str) -> &'static str {
    match code {
        "abandoned-head-element-child" => "Unexpected metadata element after head",
        "abrupt-closing-of-empty-comment" => "Unexpected abruptly closed empty comment",
        "abrupt-doctype-public-identifier" => {
            "Unexpected abruptly closed public identifier"
        }
        "abrupt-doctype-system-identifier" => {
            "Unexpected abruptly closed system identifier"
        }
        "absence-of-digits-in-numeric-character-reference" => {
            "Unexpected non-digit at start of numeric character reference"
        }
        "cdata-in-html-content" => "Unexpected CDATA section in HTML",
        "character-reference-outside-unicode-range" => {
            "Unexpected too big numeric character reference"
        }
        "closing-of-element-with-open-child-elements" => {
            "Unexpected closing tag with open child elements"
        }
        "control-character-in-input-stream" => "Unexpected control character",
        "control-character-reference" => "Unexpected control character reference",
        "disallowed-content-in-noscript-in-head" => {
            "Disallowed content inside `<noscript>` in `<head>`"
        }
        "duplicate-attribute" => "Unexpected duplicate attribute",
        "end-tag-with-attributes" => "Unexpected attribute on closing tag",
        "end-tag-with-trailing-solidus" => "Unexpected slash at end of closing tag",
        "end-tag-without-matching-open-element" => "Unexpected unopened end tag",
        "eof-before-tag-name" => "Unexpected end of file",
        "eof-in-cdata" => "Unexpected end of file in CDATA",
        "eof-in-comment" => "Unexpected end of file in comment",
        "eof-in-doctype" => "Unexpected end of file in doctype",
        "eof-in-element-that-can-contain-only-text" => {
            "Unexpected end of file in element that can only contain text"
        }
        "eof-in-script-html-comment-like-text" => {
            "Unexpected end of file in comment inside script"
        }
        "eof-in-tag" => "Unexpected end of file in tag",
        "incorrectly-closed-comment" => "Incorrectly closed comment",
        "incorrectly-opened-comment" => "Incorrectly opened comment",
        "invalid-character-sequence-after-doctype-name" => {
            "Invalid sequence after doctype name"
        }
        "invalid-first-character-of-tag-name" => "Invalid first character in tag name",
        "misplaced-doctype" => "Misplaced doctype",
        "misplaced-start-tag-for-head-element" => "Misplaced `<head>` start tag",
        "missing-attribute-value" => "Missing attribute value",
        "missing-doctype" => "Missing doctype before other content",
        "missing-doctype-name" => "Missing doctype name",
        "missing-doctype-public-identifier" => "Missing public identifier in doctype",
        "missing-doctype-system-identifier" => "Missing system identifier in doctype",
        "missing-end-tag-name" => "Missing name in end tag",
        "missing-quote-before-doctype-public-identifier" => {
            "Missing quote before public identifier in doctype"
        }
        "missing-quote-before-doctype-system-identifier" => {
            "Missing quote before system identifier in doctype"
        }
        "missing-semicolon-after-character-reference" => {
            "Missing semicolon after character reference"
        }
        "missing-whitespace-after-doctype-public-keyword" => {
            "Missing whitespace after public identifier in doctype"
        }
        "missing-whitespace-after-doctype-system-keyword" => {
            "Missing whitespace after system identifier in doctype"
        }
        "missing-whitespace-before-doctype-name" => {
            "Missing whitespace before doctype name"
        }
        "missing-whitespace-between-attributes" => {
            "Missing whitespace between attributes"
        }
        "missing-whitespace-between-doctype-public-and-system-identifiers" => {
            "Missing whitespace between public and system identifiers in doctype"
        }
        "nested-comment" => "Unexpected nested comment",
        "nested-noscript-in-head" => "Unexpected nested `<noscript>` in `<head>`",
        "non-conforming-doctype" => "Unexpected non-conforming doctype declaration",
        "non-void-html-element-start-tag-with-trailing-solidus" => {
            "Unexpected trailing slash on start tag of non-void element"
        }
        "noncharacter-character-reference" => {
            "Unexpected noncharacter code point referenced by character reference"
        }
        "noncharacter-in-input-stream" => "Unexpected noncharacter character",
        "null-character-reference" => {
            "Unexpected NULL character referenced by character reference"
        }
        "open-elements-left-after-eof" => "Unexpected end of file",
        "surrogate-character-reference" => {
            "Unexpected surrogate character referenced by character reference"
        }
        "surrogate-in-input-stream" => "Unexpected surrogate character",
        "unexpected-character-after-doctype-system-identifier" => {
            "Invalid character after system identifier in doctype"
        }
        "unexpected-character-in-attribute-name" => {
            "Unexpected character in attribute name"
        }
        "unexpected-character-in-unquoted-attribute-value" => {
            "Unexpected character in unquoted attribute value"
        }
        "unexpected-equals-sign-before-attribute-name" => {
            "Unexpected equals sign before attribute name"
        }
        "unexpected-null-character" => "Unexpected NULL character",
        "unexpected-question-mark-instead-of-tag-name" => {
            "Unexpected question mark instead of tag name"
        }
        "unexpected-solidus-in-tag" => "Unexpected slash in tag",
        "unknown-named-character-reference" => {
            "Unexpected unknown named character reference"
        }
        _ => "Unexpected parse error",
    }
}
