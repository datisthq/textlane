use serde::Deserialize;

/// Options of `md::parse`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParseOptions {}

/// Options of `md::to_html`, named as remark-rehype names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ToHtmlOptions {
    /// Keep raw HTML in the tree instead of dropping it.
    pub allow_dangerous_html: bool,
}

/// Options of `md::stringify`, named as mdast-util-to-markdown names them.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StringifyOptions {
    /// The marker of unordered list items: `*`, `+` or `-`.
    pub bullet: Option<String>,
    /// The marker of ordered list items: `.` or `)`.
    pub bullet_ordered: Option<String>,
    /// The marker of a list right after a list with `bullet`.
    pub bullet_other: Option<String>,
    /// Close ATX headings with as many `#` as they open with.
    pub close_atx: bool,
    /// The marker of emphasis: `*` or `_`.
    pub emphasis: Option<String>,
    /// The marker of fenced code: `` ` `` or `~`.
    pub fence: Option<String>,
    /// Fence code; `false` indents it where it can.
    pub fences: bool,
    /// Number ordered list items up from `start`.
    pub increment_list_marker: bool,
    /// How list item content is indented: `one`, `tab` or `mixed`.
    pub list_item_indent: Option<String>,
    /// The marker of titles: `"` or `'`.
    pub quote: Option<String>,
    /// Write links as resources even where an autolink would do.
    pub resource_link: bool,
    /// The marker of thematic breaks: `*`, `-` or `_`.
    pub rule: Option<String>,
    /// How many markers a thematic break has, 3 or more.
    pub rule_repetition: Option<i64>,
    /// Put spaces between the markers of a thematic break.
    pub rule_spaces: bool,
    /// Write headings of rank 1 and 2 as setext headings.
    pub setext: bool,
    /// The marker of strong: `*` or `_`.
    pub strong: Option<String>,
    /// Join definitions with no blank line between them.
    pub tight_definitions: bool,
    /// Extra patterns of characters to escape.
    #[serde(rename = "unsafe")]
    pub unsafe_patterns: Vec<UnsafePattern>,
}

impl Default for StringifyOptions {
    fn default() -> Self {
        StringifyOptions {
            bullet: None,
            bullet_ordered: None,
            bullet_other: None,
            close_atx: false,
            emphasis: None,
            fence: None,
            fences: true,
            increment_list_marker: true,
            list_item_indent: None,
            quote: None,
            resource_link: false,
            rule: None,
            rule_repetition: None,
            rule_spaces: false,
            setext: false,
            strong: None,
            tight_definitions: false,
            unsafe_patterns: Vec::new(),
        }
    }
}

/// A character to escape where it would otherwise start or end syntax.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UnsafePattern {
    /// The character, escaped as a regular expression would need it.
    pub character: String,
    /// A regular expression that must match right before the character.
    pub before: Option<String>,
    /// A regular expression that must match right after the character.
    pub after: Option<String>,
    /// Only at the start of a line, after any indent.
    pub at_break: bool,
    /// Only inside one of these constructs.
    pub in_construct: Option<Names>,
    /// Never inside one of these constructs.
    pub not_in_construct: Option<Names>,
    /// Only at the start of a node right after one of these node types.
    pub before_node: Option<Names>,
    /// Only at the end of a node right before one of these node types.
    pub after_node: Option<Names>,
    /// Read `before` and `after` with Unicode classes such as `\p{L}`.
    pub unicode: bool,
}

/// One name or several.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Names {
    One(String),
    Many(Vec<String>),
}

impl Names {
    /// The names as a slice.
    pub fn as_slice(&self) -> &[String] {
        match self {
            Names::One(name) => std::slice::from_ref(name),
            Names::Many(names) => names,
        }
    }
}
