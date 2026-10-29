use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Options of `md::parse`.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct ParseOptions {}

/// Options of `md::to_html`, named as remark-rehype names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct ToHtmlOptions {
    /// Keep raw HTML in the tree instead of dropping it.
    pub allow_dangerous_html: bool,
    /// What footnote `id`s start with, so they cannot clobber the page's; `user-content-`
    /// when unset.
    pub clobber_prefix: Option<String>,
    /// The text of each footnote's back reference; `↩`, numbered from the second, when
    /// unset.
    pub footnote_back_content: Option<String>,
    /// The `aria-label` of each footnote's back reference; `Back to reference 1` and so
    /// on when unset.
    pub footnote_back_label: Option<String>,
    /// The heading of the footnotes section; `Footnotes` when unset.
    pub footnote_label: Option<String>,
    /// The tag name of the footnotes section's heading; `h2` when unset.
    pub footnote_label_tag_name: Option<String>,
    /// Node types without a handler to copy into hast as they are, their children
    /// converted.
    pub pass_through: Vec<String>,
}

/// Options of `md::to_prose`, named as remark-retext and mdast-util-to-nlcst name them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
pub struct ToProseOptions {
    /// Node types to leave out, besides `table`, `tableRow` and `tableCell`.
    pub ignore: Vec<String>,
    /// Node types to keep as one `SourceNode` of their source text, besides `inlineCode`.
    pub source: Vec<String>,
}

/// Options of `md::stringify`, named as mdast-util-to-markdown names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

/// Options of `md::gfm`, named as remark-gfm names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct GfmOptions {
    /// Let one tilde, `~a~`, form strikethrough, as well as two.
    pub single_tilde: bool,
    /// Pad table cells with a space on each side when writing Markdown.
    pub table_cell_padding: bool,
    /// Line up table pipes when writing Markdown.
    pub table_pipe_align: bool,
    /// Write a footnote definition's first line blank when it holds flow content.
    pub first_line_blank: bool,
}

impl Default for GfmOptions {
    fn default() -> Self {
        GfmOptions {
            single_tilde: true,
            table_cell_padding: true,
            table_pipe_align: true,
            first_line_blank: false,
        }
    }
}

/// What `md::gfm` shares with the parser and serializer: `Some` once GFM is used.
#[derive(Clone, Debug, Default)]
pub struct GfmSlot(pub Option<GfmOptions>);

/// What `md::definition_list` shares with the parser and serializer: `true` once it is used.
#[derive(Clone, Debug, Default)]
pub struct DefinitionListSlot(pub bool);
/// Options of `md::math`, named as remark-math names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct MathOptions {
    /// Let one dollar, `$a$`, form math in text, as well as two.
    pub single_dollar_text_math: bool,
}

impl Default for MathOptions {
    fn default() -> Self {
        MathOptions {
            single_dollar_text_math: true,
        }
    }
}

/// Options of `md::directive`, named as mdast-util-directive's serializer names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct DirectiveOptions {
    /// Write an empty attribute as its name alone, `{a}`, rather than `{a=""}`.
    pub collapse_empty_attributes: bool,
    /// Write `id` and `class` as `#a` and `.b` where they can be.
    pub prefer_shortcut: bool,
    /// Leave attribute values unquoted where they can be.
    pub prefer_unquoted: bool,
    /// The quote of attribute values: `"` or `'`; `md::stringify`'s `quote` when unset.
    pub quote: Option<String>,
    /// Use the other quote when the preferred one occurs more often in a value.
    pub quote_smart: bool,
}

impl Default for DirectiveOptions {
    fn default() -> Self {
        DirectiveOptions {
            collapse_empty_attributes: true,
            prefer_shortcut: true,
            prefer_unquoted: false,
            quote: None,
            quote_smart: false,
        }
    }
}

/// What `md::math` shares with the parser and serializer: `Some` once math is used.
#[derive(Clone, Debug, Default)]
pub struct MathSlot(pub Option<MathOptions>);

/// Options of `md::mdx`, named as remark-mdx and mdast-util-mdx-jsx's serializer name them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct MdxOptions {
    /// The quote of JSX attribute values: `"` (when unset) or `'`.
    pub quote: Option<String>,
    /// Use the other quote when the preferred one occurs more often in a value.
    pub quote_smart: bool,
    /// Write self-closing tags without a space before the slash, `<a/>`.
    pub tight_self_closing: bool,
    /// The width past which a flow element's attributes go on lines of their own.
    pub print_width: Option<f64>,
}

/// Whether `md::mdx` is in the pipeline, for `md::to_html`, which may attach before it.
#[derive(Clone, Debug, Default)]
pub(crate) struct UnravelSlot(pub(crate) Arc<AtomicBool>);

/// What `md::mdx` shares with the parser and serializer: `Some` once MDX is used.
#[derive(Clone, Debug, Default)]
pub struct MdxSlot(pub Option<MdxOptions>);

/// What `md::directive` shares with the parser and serializer: `Some` once directives are
/// used.
#[derive(Clone, Debug, Default)]
pub struct DirectiveSlot(pub Option<DirectiveOptions>);

/// Options of `md::frontmatter`: the kinds of matter to recognize, `yaml` when unset.
///
/// From JSON they read as remark-frontmatter's: a preset (`"yaml"`, `"toml"`), a matter
/// (`{type, marker | fence, anywhere?}`, where `marker` is repeated three times and either
/// may be `{open, close}`), or a list of them.
#[derive(Clone, Debug)]
pub struct FrontmatterOptions(pub Vec<Matter>);

impl Default for FrontmatterOptions {
    fn default() -> Self {
        FrontmatterOptions(vec![Matter::yaml()])
    }
}

/// One kind of frontmatter: the node `kind` it becomes and the fences around it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matter {
    /// The node's `type`.
    pub kind: String,
    pub open: String,
    pub close: String,
    /// Recognize it anywhere in the document, not only on its first line.
    pub anywhere: bool,
}

impl Matter {
    /// YAML between `---` fences.
    pub fn yaml() -> Matter {
        Matter::with_marker("yaml", "-", "-")
    }

    /// TOML between `+++` fences.
    pub fn toml() -> Matter {
        Matter::with_marker("toml", "+", "+")
    }

    fn with_marker(kind: &str, open: &str, close: &str) -> Matter {
        Matter {
            kind: kind.to_string(),
            open: open.repeat(3),
            close: close.repeat(3),
            anywhere: false,
        }
    }
}

impl<'de> Deserialize<'de> for FrontmatterOptions {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        read_matters(&value)
            .map(FrontmatterOptions)
            .map_err(serde::de::Error::custom)
    }
}

// As micromark-extension-frontmatter's `toMatters`, with its messages
fn read_matters(value: &serde_json::Value) -> Result<Vec<Matter>, String> {
    use serde_json::Value;
    let options = match value {
        Value::Array(options) => options.as_slice(),
        Value::Null | Value::Bool(false) => return Ok(vec![Matter::yaml()]),
        Value::String(text) if text.is_empty() => return Ok(vec![Matter::yaml()]),
        Value::Number(number) if number.as_f64() == Some(0.0) => {
            return Ok(vec![Matter::yaml()]);
        }
        _ => std::slice::from_ref(value),
    };
    options.iter().map(read_matter).collect()
}

fn read_matter(value: &serde_json::Value) -> Result<Matter, String> {
    use serde_json::Value;
    let json = serde_json::to_string(value).unwrap_or_default();
    let object = match value {
        Value::String(preset) => {
            return match preset.as_str() {
                "yaml" => Ok(Matter::yaml()),
                "toml" => Ok(Matter::toml()),
                _ => Err(format!("Missing matter definition for `{preset}`")),
            };
        }
        Value::Object(object) => object,
        Value::Array(_) | Value::Null => {
            return Err(format!("Missing `type` in matter `{json}`"));
        }
        _ => return Err(format!("Expected matter to be an object, not `{json}`")),
    };
    let Some(kind) = object.get("type") else {
        return Err(format!("Missing `type` in matter `{json}`"));
    };
    let kind = kind
        .as_str()
        .map_or_else(|| kind.to_string(), str::to_string);
    let marker = object.get("marker").filter(|marker| is_truthy(marker));
    let (schema, repeat) = match (marker, object.get("fence")) {
        (Some(marker), _) => (marker, 3),
        (None, Some(fence)) => (fence, 1),
        (None, None) if object.contains_key("marker") => (&Value::Null, 1),
        (None, None) => {
            return Err(format!("Missing `marker` or `fence` in matter `{json}`"));
        }
    };
    let pick = |prop: &str| {
        let text = match schema {
            Value::String(text) => text.as_str(),
            Value::Object(info) => info.get(prop).and_then(Value::as_str).unwrap_or(""),
            _ => "",
        };
        text.repeat(repeat)
    };
    Ok(Matter {
        kind,
        open: pick("open"),
        close: pick("close"),
        anywhere: object.get("anywhere").is_some_and(is_truthy),
    })
}

fn is_truthy(value: &serde_json::Value) -> bool {
    use serde_json::Value;
    match value {
        Value::Null | Value::Bool(false) => false,
        Value::String(text) => !text.is_empty(),
        Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0 && !n.is_nan()),
        _ => true,
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for FrontmatterOptions {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FrontmatterOptions".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let matter = generator.subschema_for::<Matter>();
        let preset = serde_json::json!({ "enum": ["yaml", "toml"] });
        schemars::json_schema!({
            "description": "The kinds of matter to recognize, as remark-frontmatter takes them: a preset, a matter, or a list of them; `yaml` when unset.",
            "anyOf": [preset, matter, { "type": "array", "items": { "anyOf": [preset, matter] } }],
        })
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Matter {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Matter".into()
    }

    // Its fences are definitions of their own, so the generated types get names for them
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let path = generator
            .settings()
            .definitions_path
            .trim_start_matches('/')
            .to_string();
        let definitions = generator.definitions_mut();
        definitions.insert(
            "FencePair".into(),
            serde_json::json!({
                "description": "Different opening and closing fences.",
                "type": "object",
                "properties": { "open": { "type": "string" }, "close": { "type": "string" } },
                "required": ["open", "close"],
            }),
        );
        definitions.insert(
            "Fence".into(),
            serde_json::json!({
                "description": "A fence, or its opening and closing ones.",
                "anyOf": [{ "type": "string" }, { "$ref": format!("#/{path}/FencePair") }],
            }),
        );
        let fence = serde_json::json!({ "$ref": format!("#/{path}/Fence") });
        schemars::json_schema!({
            "description": "One kind of frontmatter: the node `type` it becomes and its fences, a `marker` repeated three times or a whole `fence`.",
            "type": "object",
            "properties": {
                "type": { "type": "string" },
                "marker": fence,
                "fence": fence,
                "anywhere": { "type": "boolean" },
            },
            "required": ["type"],
        })
    }
}

/// What `md::frontmatter` shares with the parser and serializer: its matters, in order.
#[derive(Clone, Debug, Default)]
pub struct FrontmatterSlot(pub Vec<Matter>);
