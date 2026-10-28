use serde::Deserialize;

/// Options of `html.stringify`, named as hast-util-to-html names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StringifyOptions {
    /// Write raw nodes as they are instead of escaping them.
    pub allow_dangerous_html: bool,
    /// Leave `'` and `` ` `` in attribute values as they are.
    pub allow_dangerous_characters: bool,
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
