use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

/// `html.stringify`: hast into HTML; options named as hast-util-to-html names them.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "StringifyOptions"))]
pub struct Stringify {
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

/// `html.parse`: HTML into hast; options named as rehype-parse names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ParseOptions"))]
pub struct Parse {
    /// Parse a fragment, as in a `template`, rather than a whole document.
    pub fragment: bool,
    /// Report parse errors as messages on the file.
    pub emit_parse_errors: bool,
    /// Add the positions of opening and closing tags and attributes to elements.
    pub verbose: bool,
}

/// `html.sanitize`: keep only what a schema allows; options: hast-util-sanitize's schema,
/// where a field left out keeps the default schema's, which is GitHub's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "SanitizeOptions"))]
pub struct Sanitize {
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum AttributeDefinition {
    Name(String),
    Values(Vec<Primitive>),
}

/// A primitive property value.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Primitive {
    Boolean(bool),
    Number(f64),
    String(String),
}

/// `html.highlight`: code blocks colored as Shiki colors them, natively by giallo;
/// options named as @shikijs/rehype names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "HighlightOptions"))]
pub struct Highlight {
    /// The theme of each color mode, `light` first, as `{"light": …, "dark": …}`.
    pub themes: Option<serde_json::Map<String, serde_json::Value>>,
    /// The one theme, when `themes` is not given.
    pub theme: Option<String>,
    /// The language of a block that names none; such a block is left alone without it.
    pub default_language: Option<String>,
    /// The language of a block naming one Shiki does not know; left alone without it.
    pub fallback_language: Option<String>,
    /// Colors to replace, by their lowercase spelling: a color for every theme, or a theme's
    /// name to a map of its own.
    pub color_replacements: BTreeMap<String, ColorReplacement>,
    /// Add `language-<name>` to each highlighted `<code>`.
    pub add_language_class: bool,
    /// Highlight inline code too, by how it names its language: `tailing-curly-colon`
    /// reads `` `a(){:js}` ``.
    pub inline: Option<InlineCode>,
    /// The `<pre>` property that gets a block's meta, the text after its language; the JSON
    /// stand-in for `parseMetaString: meta => ({ [name]: meta })`.
    pub meta_property: Option<String>,
    /// The transformers of @shikijs/transformers to run, in order, by name or with options.
    pub transformers: Vec<HighlightTransformer>,
    /// Lines at least this long, in UTF-16 units, are left one plain token as Shiki's own
    /// option leaves them; 2,000 when left out, where Shiki has no limit, since a grammar's
    /// time grows with the square of a line's length. `0` lifts it.
    pub tokenize_max_line_length: Option<usize>,
    /// Attributes of a block's `<code>` or `<pre>` listing lines, `2,4-5`, each to the
    /// classes those lines get, or to `{line, pre}`, the second for the `<pre>`:
    /// `{"data-focus": {"line": "focused", "pre": "has-focused"}}`.
    pub line_attributes: BTreeMap<String, LineMark>,
    /// The attribute of a block's `<code>` or `<pre>` listing words, separated by commas,
    /// marked at every place as `metaWordHighlight` marks a meta's `/word/`.
    pub word_attribute: Option<String>,
    /// The attribute of a block's `<code>` or `<pre>` that numbers its lines as
    /// `renderLineNumber` does, from its value, or 1 when it is empty.
    pub line_number_attribute: Option<String>,
    /// Leave a block alone when its code holds elements, such as `<mark>`, which
    /// highlighting would flatten into text.
    pub skip_elements: bool,
    /// Keep the properties of a block's `<pre>` and `<code>`, such as `data-title`, on the
    /// ones Shiki makes, merging classes; Shiki's own win where both set one.
    pub keep_properties: bool,
}

/// The classes the lines a `lineAttributes` attribute lists get, and maybe its `<pre>`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum LineMark {
    /// Classes for each line listed.
    Line(Classes),
    /// Classes for each line listed and for the `<pre>` of a block listing any.
    Both {
        line: Classes,
        #[serde(default)]
        pre: Option<Classes>,
    },
}

/// A replacement color, or a theme's own replacements.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ColorReplacement {
    Color(String),
    Theme(BTreeMap<String, String>),
}

/// How inline code names its language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum InlineCode {
    #[serde(rename = "tailing-curly-colon")]
    TailingCurlyColon,
    /// Inline `<code>` naming its language by class, `language-js`, as blocks do.
    #[serde(rename = "class")]
    Class,
}

/// A transformer of @shikijs/transformers: its name alone, or its name and options.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum HighlightTransformer {
    Name(TransformerName),
    Configured(ConfiguredTransformer),
}

/// The transformers `html.highlight` runs natively.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TransformerName {
    NotationDiff,
    NotationFocus,
    NotationHighlight,
    NotationErrorLevel,
    MetaHighlight,
    MetaWordHighlight,
    RenderLineNumber,
}

/// A transformer with its options, as @shikijs/transformers names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "name", rename_all = "camelCase")]
pub enum ConfiguredTransformer {
    /// `[!code ++]` and `[!code --]` mark added and removed lines.
    #[serde(rename_all = "camelCase")]
    NotationDiff {
        class_line_add: Option<Classes>,
        class_line_remove: Option<Classes>,
        class_active_pre: Option<Classes>,
        class_active_code: Option<Classes>,
    },
    /// `[!code focus]` marks focused lines.
    #[serde(rename_all = "camelCase")]
    NotationFocus {
        class_active_line: Option<Classes>,
        class_active_pre: Option<Classes>,
        class_active_code: Option<Classes>,
    },
    /// `[!code highlight]` and `[!code hl]` mark highlighted lines.
    #[serde(rename_all = "camelCase")]
    NotationHighlight {
        class_active_line: Option<Classes>,
        class_active_pre: Option<Classes>,
        class_active_code: Option<Classes>,
    },
    /// `[!code error]`, `[!code warning]` and `[!code info]` mark lines by level.
    #[serde(rename_all = "camelCase")]
    NotationErrorLevel {
        class_map: Option<BTreeMap<String, Classes>>,
        class_active_pre: Option<Classes>,
        class_active_code: Option<Classes>,
    },
    /// `{1,3-5}` in the meta marks highlighted lines.
    #[serde(rename_all = "camelCase")]
    MetaHighlight {
        class_name: Option<Classes>,
        #[serde(default)]
        zero_indexed: bool,
    },
    /// `/word/` in the meta marks every occurrence of the word.
    #[serde(rename_all = "camelCase")]
    MetaWordHighlight { class_name: Option<Classes> },
    /// A number at the start of each line.
    #[serde(rename_all = "camelCase")]
    RenderLineNumber {
        class_line_number: Option<Classes>,
        start: Option<usize>,
    },
}

/// Class names: a list, or one string of them separated by white space.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Classes {
    One(String),
    Many(Vec<String>),
}

/// `html.externalLinks`: `rel`, `target` and more on links to other sites, as
/// rehype-external-links adds them; options named as it names them, JSON values only.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ExternalLinksOptions"))]
pub struct ExternalLinks {
    /// The `target` of each external link, such as `_blank`.
    pub target: Option<String>,
    /// The `rel` of each external link: a list, or tokens separated by white space;
    /// `nofollow` when left out or empty, none for an empty list.
    pub rel: Option<Tokens>,
    /// The protocols of absolute URLs that count as external; `http` and `https` when left
    /// out. A protocol-relative `//…` URL always counts.
    pub protocols: Option<Vec<String>>,
    /// More properties for each external link, as hast names them.
    pub properties: Option<serde_json::Map<String, serde_json::Value>>,
    /// hast content appended to each external link in a `<span>`: one node or a list.
    pub content: Option<serde_json::Value>,
    /// The properties of that `<span>`.
    pub content_properties: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Tokens: a list, or one string of them separated by white space.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Tokens {
    One(String),
    Many(Vec<String>),
}

/// `html.katex`: math rendered as rehype-katex renders it, natively by katex-rs; options
/// named as KaTeX names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "KatexOptions"))]
pub struct Katex {
    /// The markup to write, both HTML and MathML by default.
    pub output: Option<KatexOutput>,
    /// Put equation numbers on the left.
    pub leqno: bool,
    /// Align display math to the left instead of centering it.
    pub fleqn: bool,
    /// The color of math that fails to render, `#cc0000` by default.
    pub error_color: Option<String>,
    /// Macros by name, each to the TeX it expands to, as `{"\\RR": "\\mathbb{R}"}`.
    pub macros: BTreeMap<String, String>,
    /// The thinnest a rule, such as a fraction line, is drawn, in ems.
    pub min_rule_thickness: Option<f64>,
    /// Let `\color` color the text after it, as LaTeX does.
    pub color_is_text_color: bool,
    /// What to do with input LaTeX would reject: `true` or `"error"` fails it, `"warn"`
    /// prints a warning, `false` or `"ignore"` (the default) accepts it.
    pub strict: Option<KatexStrict>,
    /// Allow commands that can run untrusted input, such as `\href` and `\includegraphics`.
    pub trust: bool,
    /// The largest a user-given size may be, in ems.
    pub max_size: Option<f64>,
    /// How many macro expansions one formula may take, 1000 by default.
    pub max_expand: Option<usize>,
    /// Let definitions such as `\def` last past the formula they are in.
    pub global_group: bool,
    /// Elements, by tag name, that hold math besides the classed ones, such as
    /// `my-math`; one is display math when it has a `display` attribute other than
    /// `inline`.
    pub tag_names: Vec<String>,
}

/// The markup KaTeX writes.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum KatexOutput {
    Html,
    Mathml,
    HtmlAndMathml,
}

/// How strictly KaTeX holds input to LaTeX.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum KatexStrict {
    Boolean(bool),
    Mode(KatexStrictMode),
}

/// One level of KaTeX's strictness.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum KatexStrictMode {
    Ignore,
    Warn,
    Error,
}

/// `html.extractToc`: the headings, nested by rank, as the file's `data.toc`, as
/// @stefanprobst/rehype-extract-toc stores them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ExtractTocOptions"))]
pub struct ExtractToc {
    /// Attributes, such as `data-toc-hidden`, that keep a heading out of the table of
    /// contents, or `name=value`, such as `data-toc=false`, for one value of it; what would
    /// nest under it nests under the heading before.
    pub skip: Vec<String>,
    /// Elements, by tag name, whose headings stay out, such as a card's title.
    pub skip_inside: Vec<String>,
    /// The shallowest heading rank listed, 1 to 6.
    pub min_depth: Option<u8>,
    /// The deepest heading rank listed, 1 to 6.
    pub max_depth: Option<u8>,
}

/// `html.slug`: ids on headings, from their text; options named as rehype-slug names
/// them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "SlugOptions"))]
pub struct Slug {
    /// Put before each id.
    pub prefix: String,
    /// Count the ids already in the document as taken, so a generated id never repeats one
    /// a link may point at.
    pub reserve_existing: bool,
    /// The heading ranks, 1 to 6, that get ids; every heading when left out.
    pub ranks: Option<Vec<u8>>,
    /// Link each heading with an `id` to itself, as rehype-autolink-headings does: `true`, or
    /// its options.
    pub autolink: Option<Autolink>,
}

/// `html.slug`'s `autolink`: on, or on with rehype-autolink-headings' options.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Autolink {
    Enabled(bool),
    Options(Box<AutolinkOptions>),
}

/// rehype-autolink-headings' options, JSON values only.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct AutolinkOptions {
    /// Where the link goes: in the heading first or last, around its content, or before or
    /// after it.
    pub behavior: AutolinkBehavior,
    /// The link's properties, as hast names them; `ariaHidden` and `tabIndex: -1` for
    /// `prepend` and `append` when left out.
    pub properties: Option<serde_json::Map<String, serde_json::Value>>,
    /// hast content of the link, one node or a list; `<span class="icon icon-link">` when
    /// left out, except for `wrap`, which appends it to the heading's content.
    pub content: Option<serde_json::Value>,
    /// A hast element wrapping the heading and its link, for `before` and `after`.
    pub group: Option<serde_json::Value>,
    /// Properties set on each linked heading.
    pub heading_properties: Option<serde_json::Map<String, serde_json::Value>>,
    /// The tag names of the headings to link; every heading when left out.
    pub test: Option<TagNames>,
}

/// Where `autolink` puts each heading's link.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum AutolinkBehavior {
    #[default]
    Prepend,
    Append,
    Wrap,
    Before,
    After,
}

/// One tag name or several.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum TagNames {
    One(String),
    Many(Vec<String>),
}

/// `html.toMarkdown`: hast into mdast; options named as hast-util-to-mdast names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ToMarkdownOptions"))]
pub struct ToMarkdown {
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
    /// Turn MathML `<math>` holding a TeX annotation, as `html.katex` writes it, into math
    /// nodes, `$…$` once `md.math` writes them, display math for `display="block"`, and
    /// drop KaTeX's visual `katex-html` copy.
    pub math: bool,
    /// Attributes of a code block's `<pre>` or `<code>` written into its meta, each to the
    /// key it takes there: `{"data-title": "title"}` makes `title="a.js"`.
    pub code_meta: BTreeMap<String, String>,
}

/// `html.raw`: reparse raw HTML and fix nesting through the HTML parser; options named as
/// hast-util-raw names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "RawOptions"))]
pub struct Raw {
    /// Node types, unknown to hast, to copy through as they are; their children are
    /// reparsed on their own.
    pub pass_through: Vec<String>,
    /// Escape the opening `<` of the tags GFM's tagfilter disallows in raw nodes.
    pub tagfilter: bool,
}

/// `html.format`: indent and collapse whitespace for readable HTML; options named as
/// hast-util-format names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "FormatOptions"))]
pub struct Format {
    /// Tag names of elements to put a blank line between, when both are one.
    pub blanks: Vec<String>,
    /// One level of indentation: a number of spaces or the text itself; two spaces when
    /// left out.
    pub indent: Option<Indent>,
    /// Indent the first level, usually `<html>`'s `head` and `body`; `true` when left out.
    pub indent_initial: Option<bool>,
    /// Only collapse whitespace, as rehype-minify-whitespace does, adding no line endings or
    /// indentation; `{ newlines: true }` collapses a run holding a line ending to it.
    pub minify: Option<Minify>,
}

/// `html.format`'s `minify`: on, or on with rehype-minify-whitespace's options.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Minify {
    Enabled(bool),
    Options(MinifyOptions),
}

/// rehype-minify-whitespace's options.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct MinifyOptions {
    /// Collapse a run of whitespace holding a line ending to a line ending, not a space.
    pub newlines: bool,
}

impl Format {
    /// Whether to only minify, and with `newlines`.
    pub(crate) fn minify_newlines(&self) -> Option<bool> {
        match &self.minify {
            Some(Minify::Enabled(true)) => Some(false),
            Some(Minify::Options(options)) => Some(options.newlines),
            Some(Minify::Enabled(false)) | None => None,
        }
    }
}

/// One level of indentation.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Indent {
    Spaces(usize),
    Text(String),
}

/// The elements prose plugins leave alone when `ignore` is left out: links, code, scripts,
/// form text and foreign content.
pub const PROSE_IGNORE: &[&str] = &[
    "a", "code", "pre", "kbd", "samp", "script", "style", "textarea", "title", "math",
    "svg",
];

/// `html.math`: `$…$` and `$$…$$` in text into the elements `html.katex` renders, read by
/// pandoc's rule so prices stay text: an opening `$` is followed by a non-space, a closing
/// one follows a non-space and is not followed by a digit; `\$` is a dollar sign.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "MathOptions"))]
pub struct Math {
    /// Let one dollar, `$a$`, form math, as well as two.
    pub single_dollar_text_math: bool,
    /// Elements, by tag name, whose text is left alone; links, code, scripts, form text,
    /// `math` and `svg` when left out.
    pub ignore: Option<Vec<String>>,
    /// Attributes, such as `data-literal`, whose elements' text is left alone.
    pub ignore_attributes: Vec<String>,
}

impl Default for Math {
    fn default() -> Self {
        Math {
            single_dollar_text_math: true,
            ignore: None,
            ignore_attributes: Vec::new(),
        }
    }
}

/// `html.linkify`: bare URLs and email addresses in text into links, as GitHub's literal
/// autolinks do: a URL with `://` as it is, a `www.` one under `https://`, an email under
/// `mailto:`.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "LinkifyOptions"))]
pub struct Linkify {
    /// Elements, by tag name, whose text is left alone; links, code, scripts, form text,
    /// `math` and `svg` when left out.
    pub ignore: Option<Vec<String>>,
    /// Attributes, such as `data-literal`, whose elements' text is left alone.
    pub ignore_attributes: Vec<String>,
}

/// `html.gemoji`: gemoji shortcodes in text, `:smile:`, into emoji, as `md.gemoji` does in
/// Markdown.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "GemojiOptions"))]
pub struct Gemoji {
    /// Wrap each emoji in `<span role="img" aria-label="…">`, labelled with its name.
    pub accessible: bool,
    /// Elements, by tag name, whose text is left alone; links, code, scripts, form text,
    /// `math` and `svg` when left out.
    pub ignore: Option<Vec<String>>,
    /// Attributes, such as `data-literal`, whose elements' text is left alone.
    pub ignore_attributes: Vec<String>,
}

/// `html.pathRewrite`: the URLs of a document's attributes as a site serves them, as
/// `md.pathRewrite` does for Markdown: relative ones resolved against the file's path and
/// made root-relative, every root-relative one under `base`, and links to a page's file
/// turned into its URL.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "PathRewriteOptions"))]
pub struct PathRewrite {
    /// The directory URLs count from, as the file's path spells it; the path's own
    /// starting point when left out.
    pub root: Option<String>,
    /// The URL the site is served at, such as `/docs`, put before every root-relative URL
    /// once.
    pub base: Option<String>,
    /// Files, relative to `root`, to the URL of their page, such as
    /// `{"guide.html": "/guide/"}`; an `href` naming one takes it.
    pub pages: BTreeMap<String, String>,
    /// The attributes whose URLs are rewritten; `href`, `src`, `srcset` and `poster` when
    /// left out.
    pub attributes: Option<Vec<String>>,
}
