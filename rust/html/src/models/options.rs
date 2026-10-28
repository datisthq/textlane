use serde::Deserialize;

/// Options of `html.stringify`, named as hast-util-to-html names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StringifyOptions {
    /// Write raw nodes as they are instead of escaping them.
    pub allow_dangerous_html: bool,
    /// Leave characters that are only dangerous in some contexts, such as `` ` ``, as they are.
    pub allow_dangerous_characters: bool,
    /// Leave characters as they are where browsers accept them though the spec does not.
    pub allow_parse_errors: bool,
    /// Write comments as bogus comments, `<?a>`.
    pub bogus_comments: bool,
    /// Close empty foreign elements, `<a />`.
    pub close_empty_elements: bool,
    /// Close void elements, `<br />`.
    pub close_self_closing: bool,
    /// Write empty attributes as bare names, `a` rather than `a=""`.
    pub collapse_empty_attributes: bool,
    /// Leave out tags the HTML spec lets a parser imply.
    pub omit_optional_tags: bool,
    /// Leave attribute values unquoted where that is safe.
    pub prefer_unquoted: bool,
    /// The quote attribute values are wrapped in: `"` or `'`.
    pub quote: Option<String>,
    /// Use the other quote when the value holds more of the preferred one.
    pub quote_smart: bool,
    /// The space the tree is in: `html` or `svg`.
    pub space: Option<String>,
    /// Leave out the space between quoted attributes.
    pub tight_attributes: bool,
    /// Join comma-separated values with `,` rather than `, `.
    pub tight_comma_separated_lists: bool,
    /// Write `<!doctypehtml>`.
    pub tight_doctype: bool,
    /// Leave out the space before `/>` when the attributes allow.
    pub tight_self_closing: bool,
    /// Write `<!DOCTYPE html>`.
    pub upper_doctype: bool,
    /// The tag names of void elements, which have no closing tag.
    pub voids: Option<Vec<String>>,
}

/// Options of `html.parse`, named as rehype-parse names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParseOptions {
    /// Parse a fragment, as in a `template`, rather than a whole document.
    pub fragment: bool,
    /// Report parse errors as messages on the file.
    pub emit_parse_errors: bool,
    /// Add the positions of opening and closing tags and attributes to elements.
    pub verbose: bool,
}
