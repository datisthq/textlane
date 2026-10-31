use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// `md.parse`: Markdown into mdast.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ParseOptions"))]
pub struct Parse {}

/// `md.toHtml`: mdast into hast; options named as remark-rehype names them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ToHtmlOptions"))]
pub struct ToHtml {
    /// Keep raw HTML in the tree instead of dropping it.
    pub allow_dangerous_html: bool,
    /// Turn each line ending in text into a `<br>`, as remark-breaks does.
    pub breaks: bool,
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
    /// An `aria-label` for each task-list checkbox, so screen readers name it; none when
    /// left out, as remark-rehype writes none.
    pub task_list_labels: Option<TaskListLabels>,
}

/// The labels of task-list checkboxes.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TaskListLabels {
    /// The label of a checked box, such as `Completed task`.
    pub checked: String,
    /// The label of an unchecked box, such as `Task`.
    pub unchecked: String,
}

/// `md.toNaturalLanguage`: mdast into nlcst, tokenized by the processor's natural-language parser; options
/// named as remark-retext and mdast-util-to-nlcst name them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
#[cfg_attr(feature = "schema", schemars(rename = "ToNaturalLanguageOptions"))]
pub struct ToNaturalLanguage {
    /// Node types to leave out, besides `table`, `tableRow` and `tableCell`.
    pub ignore: Vec<String>,
    /// Node types to keep as one `SourceNode` of their source text, besides `inlineCode`.
    pub source: Vec<String>,
}

/// `md.stringify`: mdast into Markdown; options named as mdast-util-to-markdown names
/// them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "StringifyOptions"))]
pub struct Stringify {
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

impl Default for Stringify {
    fn default() -> Self {
        Stringify {
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

impl Default for Names {
    fn default() -> Self {
        Names::Many(Vec::new())
    }
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

/// `md.gfm`: GitHub Flavored Markdown's syntax, read by `md.parse` and written by
/// `md.stringify`; options named as remark-gfm names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "GfmOptions"))]
pub struct Gfm {
    /// Let one tilde, `~a~`, form strikethrough, as well as two.
    pub single_tilde: bool,
    /// Pad table cells with a space on each side when writing Markdown.
    pub table_cell_padding: bool,
    /// Line up table pipes when writing Markdown.
    pub table_pipe_align: bool,
    /// Write a footnote definition's first line blank when it holds flow content.
    pub first_line_blank: bool,
    /// Render GitHub alerts, a block quote opening with `[!NOTE]`, as GitHub does, as
    /// remark-github-blockquote-alert does: `true`, or its options.
    pub alerts: Option<Alerts>,
}

/// `md.gfm`'s `alerts`: on, or on with remark-github-blockquote-alert's options.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Alerts {
    Enabled(bool),
    Options(AlertsOptions),
}

/// remark-github-blockquote-alert's options.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct AlertsOptions {
    /// Read a title after the type, `[!NOTE/Read this]`, in place of the type's name.
    pub legacy_title: bool,
    /// The tag name of the alert's element.
    pub tag_name: String,
    /// More classes for the alert's element, separated by spaces.
    pub class_names: String,
}

impl Default for AlertsOptions {
    fn default() -> Self {
        AlertsOptions {
            legacy_title: false,
            tag_name: "div".to_string(),
            class_names: String::new(),
        }
    }
}

impl Default for Gfm {
    fn default() -> Self {
        Gfm {
            single_tilde: true,
            table_cell_padding: true,
            table_pipe_align: true,
            first_line_blank: false,
            alerts: None,
        }
    }
}

/// What `md::Gfm` shares with the parser and serializer: `Some` once GFM is used.
#[derive(Clone, Debug, Default)]
pub struct GfmSlot(pub Option<Gfm>);

/// What `md::DefinitionList` shares with the parser and serializer: `true` once it is used.
#[derive(Clone, Debug, Default)]
pub struct DefinitionListSlot(pub bool);
/// `md.math`: math syntax, `$$` blocks and `$` text, read by `md.parse` and written by
/// `md.stringify`; options named as remark-math names them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "MathOptions"))]
pub struct Math {
    /// Let one dollar, `$a$`, form math in text, as well as two.
    pub single_dollar_text_math: bool,
}

impl Default for Math {
    fn default() -> Self {
        Math {
            single_dollar_text_math: true,
        }
    }
}

/// `md.headingId`: `{#id}` at the end of a heading, after a space, as its id, as
/// remark-heading-id reads it; headings without one keep none.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "HeadingIdOptions"))]
pub struct HeadingId {
    /// Read a whole attribute list, `{#id .class key=value key="v w"}`, as directives
    /// write theirs: classes become `className` and other keys properties.
    pub attributes: bool,
    /// Literal suffixes, such as `[step]`, stripped from a heading's end for the attributes
    /// each names, a value or `true` for a bare one: `{"[step]": {"data-step": true}}`.
    pub markers: BTreeMap<String, BTreeMap<String, MarkerValue>>,
}

/// An attribute a heading marker sets: its value, or `true` for the bare attribute.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum MarkerValue {
    Flag(bool),
    Text(String),
}

/// `md.shiftHeading`: every heading's depth moved by `shift` and kept within 1 to 6, as
/// rehype-shift-heading moves hast's, after the document's title is dropped when asked.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ShiftHeadingOptions"))]
pub struct ShiftHeading {
    /// The levels added to each heading's depth: `-1` makes an h2 an h1, `1` an h1 an h2.
    pub shift: i32,
    /// Drop the document's first node when it is an h1, looking past frontmatter and ESM.
    pub drop_title: bool,
}

/// `md.pathRewrite`: the URLs of images, links and definitions as a site serves them:
/// relative ones resolved against the file's path and made root-relative, every root-relative
/// one under `base`, and links to Markdown files turned into page URLs.
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
    /// The nodes whose URLs are rewritten; images, links and definitions when left out.
    pub nodes: Option<Vec<UrlNode>>,
    /// Markdown files, relative to `root`, to the URL of their page; a link to one of them
    /// takes it, before `markdownLinks` applies.
    pub pages: BTreeMap<String, String>,
    /// How a link to a Markdown file no page names becomes a URL.
    pub markdown_links: MarkdownLinks,
}

/// A node with a URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum UrlNode {
    Image,
    Link,
    Definition,
}

/// How a link to a Markdown file becomes a URL.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum MarkdownLinks {
    /// The file's own path, `/guide/intro.md`.
    #[default]
    Keep,
    /// A directory, `/guide/intro/`, an `index` file its parent's.
    Directory,
    /// An HTML page, `/guide/intro.html`.
    Html,
}

/// `md.strip`: mdast down to its text, as strip-markdown does: code, HTML, tables and
/// the like dropped, headings made paragraphs, formatting unwrapped.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "StripOptions"))]
pub struct Strip {
    /// Node types to leave as they are, such as `inlineCode`.
    pub keep: Vec<String>,
    /// Node types to drop, besides those strip-markdown drops.
    pub remove: Vec<String>,
}

/// `md.toc`: a table of contents under the first heading matching `heading`, replacing
/// what sat between it and the next heading as deep or shallower, as remark-toc makes it.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "TocOptions"))]
pub struct Toc {
    /// The heading to put the table under, as JavaScript regular expression source matched
    /// against its whole text, case-insensitively; `(table[ -]of[ -])?contents?|toc` when
    /// unset.
    pub heading: Option<String>,
    /// The deepest heading rank to list, 1 to 6.
    pub max_depth: Option<u8>,
    /// The shallowest heading rank to list, 1 to 6.
    pub min_depth: Option<u8>,
    /// Headings to leave out, as JavaScript regular expression source matched against
    /// their whole text, case-insensitively.
    pub skip: Option<String>,
    /// Keep the list compact, without blank lines between items; `true` when unset.
    pub tight: Option<bool>,
    /// Number the list instead of bulleting it.
    pub ordered: Option<bool>,
    /// What each link's fragment starts with, such as `user-content-`.
    pub prefix: Option<String>,
    /// The parents whose headings are listed, as a unist-util-is test: a node type, an
    /// object of fields a parent must equal, or a list of these; only the root when unset.
    pub parents: Option<ParentTest>,
}

/// A unist-util-is test of a node, written as JSON: one check, or a list any of which
/// passes.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ParentTest {
    /// One check.
    One(ParentCheck),
    /// A node passing any of these checks.
    Any(Vec<ParentCheck>),
}

/// One unist-util-is check: a node type, or fields a node must equal.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ParentCheck {
    /// A node of this type.
    Kind(String),
    /// A node whose fields equal these values; only strings, numbers, booleans and `null`
    /// can ever be equal.
    Fields(serde_json::Map<String, serde_json::Value>),
}

/// `md.component`: directives, GitHub alerts and other nodes into MDX JSX elements, the
/// components an MDX runtime renders, by declarative rules.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "ComponentOptions"))]
pub struct Component {
    /// Rules for directives; the first that matches a directive converts it.
    pub directives: Vec<DirectiveRule>,
    /// GitHub alerts, `> [!NOTE]`, into a component.
    pub alerts: Option<AlertRule>,
    /// Node types, such as `defList`, into components keeping their children.
    pub nodes: BTreeMap<String, String>,
    /// What becomes of a directive no rule converts.
    pub unknown_directives: UnknownDirectives,
    /// Rules for fenced code; the first that matches a block applies.
    pub codes: Vec<CodeRule>,
}

/// How `md.component` turns matching code blocks into a component, or groups them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct CodeRule {
    /// The block's language, or several; any when left out.
    pub lang: Option<Names>,
    /// A `key="value"` the block's meta must hold.
    pub meta: Option<String>,
    /// The component the block becomes; left out, the block stays code and only groups.
    pub component: Option<String>,
    /// The attribute that gets the code; left out, the code is the component's text.
    pub value_as: Option<String>,
    /// Meta `key="value"` pairs to keep as attributes, in order.
    pub attributes: Vec<AttributeRule>,
    /// Wrap runs of adjacent blocks this rule matched in another component.
    pub group: Option<CodeGroupRule>,
}

/// The component wrapping a run of adjacent code blocks one rule matched; the meta keys it
/// reads are taken out of each block's meta.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CodeGroupRule {
    /// The wrapping component.
    pub component: String,
    /// The shortest run wrapped; 2 when left out.
    #[serde(default)]
    pub min: Option<usize>,
    /// The attribute that gets the run's values of the rule's `meta` key, as a JSON list.
    #[serde(default)]
    pub collect: Option<String>,
    /// Meta keys the wrapper copies from the run's first block as attributes.
    #[serde(default)]
    pub inherit: Vec<String>,
}

/// How `md.component` turns matching directives into one component.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DirectiveRule {
    /// The directive's name, or several.
    pub name: Names,
    /// The kind of directive: `text`, `leaf` or `container`; any when left out.
    #[serde(default)]
    pub kind: Option<DirectiveKind>,
    /// Attribute values the directive must have.
    #[serde(default, rename = "where")]
    pub condition: BTreeMap<String, String>,
    /// The component: a text element for a text directive, else a flow element.
    pub component: String,
    /// The attribute that gets the directive's name.
    #[serde(default)]
    pub name_as: Option<String>,
    /// The attribute that gets the label's text: a container's label paragraph or a text
    /// or leaf directive's content, which is then dropped.
    #[serde(default)]
    pub label_as: Option<String>,
    /// The label's text when there is none.
    #[serde(default)]
    pub label_default: Option<String>,
    /// The attributes to keep, in order; every one, as written, when left out.
    #[serde(default)]
    pub attributes: Option<Vec<AttributeRule>>,
    /// Attributes without which the directive is left to `unknownDirectives`.
    #[serde(default)]
    pub required: Vec<String>,
    /// Wrap runs of adjacent siblings this rule converted in another component.
    #[serde(default)]
    pub group: Option<GroupRule>,
    /// The attribute that gets the lists of a container directive as a file tree, JSON
    /// items `{name, type, comment?, highlighted?, children?}`; the lists leave its content.
    #[serde(default)]
    pub tree_as: Option<String>,
}

/// A directive's kind, by how many colons open it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum DirectiveKind {
    Text,
    Leaf,
    Container,
}

/// An attribute to keep: its name, or its name with what to call it and a default.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum AttributeRule {
    Name(String),
    Rule {
        name: String,
        #[serde(default)]
        rename: Option<String>,
        #[serde(default)]
        default: Option<String>,
    },
}

/// The component wrapping a run of adjacent siblings one rule converted.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GroupRule {
    /// The wrapping component.
    pub component: String,
    /// The shortest run wrapped; 2 when left out.
    #[serde(default)]
    pub min: Option<usize>,
    /// Attributes the wrapper copies from the run's first element.
    #[serde(default)]
    pub inherit: Vec<String>,
}

/// How `md.component` turns GitHub alerts, a block quote opening with `[!TYPE]`, into a
/// component.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AlertRule {
    /// The component an alert becomes.
    pub component: String,
    /// The attribute that gets the alert's type.
    pub type_as: String,
    /// Each alert type, as written, to the value of `typeAs`; other types stay block quotes.
    pub types: BTreeMap<String, String>,
}

/// What `md.component` does with a directive no rule converts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum UnknownDirectives {
    /// Leave it a directive, as unified does.
    #[default]
    Keep,
    /// Put a text or leaf directive back as the text it was written as, such as the `:1` of
    /// `3:1`; container directives stay.
    Source,
}

/// `md.packageCommand`: a fenced block of package-manager commands, its language naming
/// the ecosystem (`npm`, `pip`, `cargo`), into one component with the commands converted
/// for each manager, as `@docusaurus/remark-plugin-npm2yarn` does for npm.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "PackageCommandOptions"))]
pub struct PackageCommand {
    /// The component a block becomes; one string attribute per manager.
    pub component: String,
    /// Each ecosystem's managers, in attribute order; an empty list leaves its blocks code.
    pub ecosystems: Ecosystems,
}

impl Default for PackageCommand {
    fn default() -> Self {
        PackageCommand {
            component: "PackageTabs".to_owned(),
            ecosystems: Ecosystems::default(),
        }
    }
}

/// The managers `md.packageCommand` converts each ecosystem's commands for.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct Ecosystems {
    /// Managers for `npm` blocks, converted as npm-to-yarn converts; npm, pnpm, yarn and
    /// bun by default.
    pub npm: Vec<NpmManager>,
    /// Managers for `pip` blocks; pip, uv, poetry and pdm by default.
    pub pip: Vec<PipManager>,
    /// Managers for `cargo` blocks; cargo and cargo-binstall by default.
    pub cargo: Vec<CargoManager>,
}

impl Default for Ecosystems {
    fn default() -> Self {
        Ecosystems {
            npm: vec![
                NpmManager::Npm,
                NpmManager::Pnpm,
                NpmManager::Yarn,
                NpmManager::Bun,
            ],
            pip: vec![
                PipManager::Pip,
                PipManager::Uv,
                PipManager::Poetry,
                PipManager::Pdm,
            ],
            cargo: vec![CargoManager::Cargo, CargoManager::CargoBinstall],
        }
    }
}

/// A JavaScript package manager an `npm` command converts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum NpmManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Deno,
    Nub,
}

/// A Python package manager a `pip` command converts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum PipManager {
    Pip,
    Uv,
    Poetry,
    Pdm,
}

/// A Rust installer a `cargo` command converts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum CargoManager {
    Cargo,
    CargoBinstall,
}

impl NpmManager {
    /// The manager's name, as options spell it and as its attribute is named.
    pub fn name(self) -> &'static str {
        match self {
            NpmManager::Npm => "npm",
            NpmManager::Pnpm => "pnpm",
            NpmManager::Yarn => "yarn",
            NpmManager::Bun => "bun",
            NpmManager::Deno => "deno",
            NpmManager::Nub => "nub",
        }
    }
}

impl PipManager {
    /// The manager's name, as options spell it and as its attribute is named.
    pub fn name(self) -> &'static str {
        match self {
            PipManager::Pip => "pip",
            PipManager::Uv => "uv",
            PipManager::Poetry => "poetry",
            PipManager::Pdm => "pdm",
        }
    }
}

impl CargoManager {
    /// The manager's name, as options spell it and as its attribute is named.
    pub fn name(self) -> &'static str {
        match self {
            CargoManager::Cargo => "cargo",
            CargoManager::CargoBinstall => "cargo-binstall",
        }
    }
}

/// `md.directive`: generic directives, `:a`, `::b` and `:::c`, read by `md.parse` and
/// written by `md.stringify`; options named as mdast-util-directive's serializer names
/// them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "DirectiveOptions"))]
pub struct Directive {
    /// Write an empty attribute as its name alone, `{a}`, rather than `{a=""}`.
    pub collapse_empty_attributes: bool,
    /// Write `id` and `class` as `#a` and `.b` where they can be.
    pub prefer_shortcut: bool,
    /// Leave attribute values unquoted where they can be.
    pub prefer_unquoted: bool,
    /// The quote of attribute values: `"` or `'`; `md.stringify`'s `quote` when unset.
    pub quote: Option<String>,
    /// Use the other quote when the preferred one occurs more often in a value.
    pub quote_smart: bool,
}

impl Default for Directive {
    fn default() -> Self {
        Directive {
            collapse_empty_attributes: true,
            prefer_shortcut: true,
            prefer_unquoted: false,
            quote: None,
            quote_smart: false,
        }
    }
}

/// What `md::Math` shares with the parser and serializer: `Some` once math is used.
#[derive(Clone, Debug, Default)]
pub struct MathSlot(pub Option<Math>);

/// `md.mdx`: MDX syntax, JSX, expressions and ESM, read by `md.parse` and written by
/// `md.stringify`; it turns off indented code, autolinks and raw HTML; options named as
/// remark-mdx and mdast-util-mdx-jsx's serializer name them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "MdxOptions"))]
pub struct Mdx {
    /// The quote of JSX attribute values: `"` (when unset) or `'`.
    pub quote: Option<String>,
    /// Use the other quote when the preferred one occurs more often in a value.
    pub quote_smart: bool,
    /// Write self-closing tags without a space before the slash, `<a/>`.
    pub tight_self_closing: bool,
    /// The width past which a flow element's attributes go on lines of their own.
    pub print_width: Option<f64>,
}

/// Whether `md::Mdx` is in the pipeline, for `md::ToHtml`, which may attach before it.
#[derive(Clone, Debug, Default)]
pub(crate) struct UnravelSlot(pub(crate) Arc<AtomicBool>);

/// What `md::Mdx` shares with the parser and serializer: `Some` once MDX is used.
#[derive(Clone, Debug, Default)]
pub struct MdxSlot(pub Option<Mdx>);

/// What `md::Directive` shares with the parser and serializer: `Some` once directives are
/// used.
#[derive(Clone, Debug, Default)]
pub struct DirectiveSlot(pub Option<Directive>);

/// `md.frontmatter`: YAML, TOML or custom matter at the start of a document, read by
/// `md.parse` and written by `md.stringify`; with `data`, the YAML a document starts with is
/// parsed into `file.data.matter`, as vfile-matter does.
///
/// From JSON they read as remark-frontmatter's (a preset, `"yaml"` or `"toml"`; a matter,
/// `{type, marker | fence, anywhere?}`, where `marker` is repeated three times and either
/// may be `{open, close}`; or a list of them), or as `{matters?, data?, strip?}`.
#[derive(Clone, Debug)]
pub struct Frontmatter {
    /// The kinds of matter to recognize, `yaml` when unset.
    pub matters: Vec<Matter>,
    /// Parse the YAML frontmatter a document starts with into `file.data.matter`, `{}`
    /// when there is none.
    pub data: bool,
    /// Remove the YAML frontmatter a document starts with from the tree.
    pub strip: bool,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Frontmatter {
            matters: vec![Matter::yaml()],
            data: false,
            strip: false,
        }
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

impl<'de> Deserialize<'de> for Frontmatter {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        read_frontmatter(&value).map_err(serde::de::Error::custom)
    }
}

// An object with none of a matter's keys but one of textlane's own is the options object
fn read_frontmatter(value: &serde_json::Value) -> Result<Frontmatter, String> {
    let own = value.as_object().filter(|object| {
        !["type", "marker", "fence"]
            .iter()
            .any(|key| object.contains_key(*key))
            && ["matters", "data", "strip"]
                .iter()
                .any(|key| object.contains_key(*key))
    });
    let Some(object) = own else {
        return Ok(Frontmatter {
            matters: read_matters(value)?,
            ..Frontmatter::default()
        });
    };
    let flag = |key: &str| match object.get(key) {
        None | Some(serde_json::Value::Null) => Ok(false),
        Some(serde_json::Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(format!("Expected `{key}` to be a boolean, not `{other}`")),
    };
    Ok(Frontmatter {
        matters: read_matters(object.get("matters").unwrap_or(&serde_json::Value::Null))?,
        data: flag("data")?,
        strip: flag("strip")?,
    })
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
impl schemars::JsonSchema for Frontmatter {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FrontmatterOptions".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let matter = generator.subschema_for::<Matter>();
        let preset = serde_json::json!({ "enum": ["yaml", "toml"] });
        let matters = serde_json::json!({
            "anyOf": [preset, matter, { "type": "array", "items": { "anyOf": [preset, matter] } }],
        });
        let path = generator
            .settings()
            .definitions_path
            .trim_start_matches('/')
            .to_string();
        let settings = serde_json::json!({
            "description": "textlane's options: the matters, and what to do with YAML frontmatter.",
            "type": "object",
            "properties": {
                "matters": {
                    "description": "The kinds of matter to recognize, as remark-frontmatter takes them; `yaml` when unset.",
                    "anyOf": matters["anyOf"],
                },
                "data": {
                    "description": "Parse the YAML frontmatter a document starts with into `file.data.matter`, `{}` when there is none, as vfile-matter does.",
                    "type": "boolean",
                },
                "strip": {
                    "description": "Remove the YAML frontmatter a document starts with from the tree.",
                    "type": "boolean",
                },
            },
        });
        generator
            .definitions_mut()
            .insert("FrontmatterSettings".into(), settings);
        let object =
            serde_json::json!({ "$ref": format!("#/{path}/FrontmatterSettings") });
        schemars::json_schema!({
            "description": "The kinds of matter to recognize, as remark-frontmatter takes them (a preset, a matter, or a list of them; `yaml` when unset), or textlane's options object.",
            "anyOf": [preset, matter, { "type": "array", "items": { "anyOf": [preset, matter] } }, object],
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

/// What `md::Frontmatter` shares with the parser and serializer: its matters, in order.
#[derive(Clone, Debug, Default)]
pub struct FrontmatterSlot(pub Vec<Matter>);

/// `md.smartypants`: straight quotes, dashes, dots and backticks in Markdown text into
/// typographic ones, as remark-smartypants does; options as retext-smartypants takes them.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "SmartypantsOptions"))]
pub struct Smartypants {
    /// Turn double backticks and double straight single quotes into double quotes; `all`
    /// turns single ones into single quotes too.
    pub backticks: textlane_core::Backticks,
    /// The closing quotes to use: `”` and `’` by default.
    pub closing_quotes: Option<textlane_core::QuoteCharacters>,
    /// Turn two dashes into an em dash; `oldschool` and `inverted` treat three too.
    pub dashes: textlane_core::Dashes,
    /// Turn three dots into an ellipsis, with spaces between them or without.
    pub ellipses: textlane_core::Ellipses,
    /// The opening quotes to use: `“` and `‘` by default.
    pub opening_quotes: Option<textlane_core::QuoteCharacters>,
    /// Turn straight quotes into smart ones.
    #[serde(deserialize_with = "textlane_core::read_switch")]
    #[cfg_attr(feature = "schema", schemars(with = "Option<bool>"))]
    pub quotes: bool,
}

impl Default for Smartypants {
    fn default() -> Self {
        Self {
            backticks: textlane_core::Backticks::On,
            closing_quotes: None,
            dashes: textlane_core::Dashes::On,
            ellipses: textlane_core::Ellipses::On,
            opening_quotes: None,
            quotes: true,
        }
    }
}

/// `md.gemoji`: gemoji shortcodes, `:smile:`, into emoji, as remark-gemoji does; its options
/// are remark-emoji's.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", schemars(rename = "GemojiOptions"))]
pub struct Gemoji {
    /// Also turn emoticons such as `:)` and `:-(` into emoji, as remark-emoji does.
    pub emoticon: bool,
    /// Wrap each emoji in `<span role="img" aria-label="…">`, labelled with its name.
    pub accessible: bool,
    /// Put a space after each emoji.
    pub pad_space_after: bool,
}
