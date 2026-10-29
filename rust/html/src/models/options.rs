use std::collections::{BTreeMap, HashMap};

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

/// Options of `html.sanitize`: hast-util-sanitize's schema; a field left out keeps the
/// default schema's, which is GitHub's.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SanitizeOptions {
    /// Keep comments.
    pub allow_comments: Option<bool>,
    /// Keep doctypes.
    pub allow_doctypes: Option<bool>,
    /// Elements allowed only inside one of these ancestors, by tag name.
    pub ancestors: Option<HashMap<String, Vec<String>>>,
    /// Properties allowed per tag name, `*` for every element; a list names a property and
    /// the values it may have.
    pub attributes: Option<HashMap<String, Vec<AttributeDefinition>>>,
    /// Properties whose values get `clobber_prefix`, so they cannot clobber globals.
    pub clobber: Option<Vec<String>>,
    /// The prefix of clobbered values.
    pub clobber_prefix: Option<String>,
    /// URL properties and the protocols they may use.
    pub protocols: Option<HashMap<String, Vec<String>>>,
    /// Properties set on elements whatever they had, by tag name.
    pub required: Option<HashMap<String, BTreeMap<String, Primitive>>>,
    /// Elements dropped with their content rather than unwrapped.
    pub strip: Option<Vec<String>>,
    /// The elements kept.
    pub tag_names: Option<Vec<String>>,
}

/// A property a sanitize schema allows: by name, or a name and its allowed values.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum AttributeDefinition {
    Name(String),
    Values(Vec<Primitive>),
}

/// A primitive property value.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Boolean(bool),
    Number(f64),
    String(String),
}

/// Options of `html.highlight`, named as @shikijs/rehype names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct HighlightOptions {
    /// The theme of each color mode, `light` first, as `{"light": …, "dark": …}`.
    pub themes: Option<serde_json::Map<String, serde_json::Value>>,
    /// The one theme, when `themes` is not given.
    pub theme: Option<String>,
    /// The language of a block that names none; such a block is left alone without it.
    pub default_language: Option<String>,
    /// The language of a block naming one Shiki does not know; left alone without it.
    pub fallback_language: Option<String>,
}

/// Options of `html.slug`, named as rehype-slug names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SlugOptions {
    /// Put before each id.
    pub prefix: String,
}

/// Options of `html.toMarkdown`, named as hast-util-to-mdast names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ToMarkdownOptions {
    /// Wrap all phrasing content of the root in paragraphs, as for a whole document.
    pub document: bool,
    /// Collapse whitespace holding a line ending to that line ending instead of a space.
    pub newlines: bool,
    /// The text of a checked checkbox or radio, `[x]` by default.
    pub checked: Option<String>,
    /// The text of an unchecked checkbox or radio, `[ ]` by default.
    pub unchecked: Option<String>,
    /// The quotes `<q>` uses, an opening and closing character each, alternating by how
    /// deep it nests; `"` by default.
    pub quotes: Option<Vec<String>>,
}
