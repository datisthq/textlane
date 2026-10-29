//! Textlane's Markdown plugins: parsing, syntax extensions, mdast transforms, serializing and
//! the bridge to HTML.

mod plugin;
mod models {
    pub(crate) mod code;
    pub(crate) mod construct;
    pub(crate) mod def_list;
    pub(crate) mod directive;
    pub(crate) mod event;
    pub(crate) mod mdx;
    pub(crate) mod options;
    pub(crate) mod token;
}
mod utils {
    pub(crate) mod compiler {
        pub(crate) mod handlers;
        pub(crate) mod stack;
    }
    pub(crate) mod converter {
        pub(crate) mod footnote;
        pub(crate) mod state;
        pub(crate) mod unknown;
    }
    pub(crate) mod serializer {
        pub(crate) mod construct;
        pub(crate) mod format;
        pub(crate) mod join;
        pub(crate) mod phrasing;
        pub(crate) mod safe;
        pub(crate) mod settings;
        pub(crate) mod sink;
        pub(crate) mod state;
        pub(crate) mod unsafe_patterns;
    }
    pub(crate) mod tokenizer {
        pub(crate) mod content;
        pub(crate) mod document;
        pub(crate) mod driver;
        pub(crate) mod edit_map;
        pub(crate) mod flow;
        pub(crate) mod preprocess;
        pub(crate) mod registry;
        pub(crate) mod string;
        pub(crate) mod subtokenize;
        pub(crate) mod text;
    }
}
mod actions {
    pub(crate) mod attention {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod autolink {
        pub(crate) mod tokenize;
    }
    pub(crate) mod blank_line {
        pub(crate) mod tokenize;
    }
    pub(crate) mod block_quote {
        pub(crate) mod tokenize;
    }
    pub(crate) mod blockquote {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod character_escape {
        pub(crate) mod tokenize;
    }
    pub(crate) mod character_reference {
        pub(crate) mod compile;
        pub(crate) mod tokenize;
    }
    pub(crate) mod code {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod code_fenced {
        pub(crate) mod tokenize;
    }
    pub(crate) mod code_indented {
        pub(crate) mod tokenize;
    }
    pub(crate) mod code_text {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod data {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod def_list {
        pub(crate) mod check;
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod resolve;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
        pub(crate) mod track;
    }
    pub(crate) mod def_list_description {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod def_list_term {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod definition {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod delete {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod destination {
        pub(crate) mod tokenize;
    }
    pub(crate) mod directive {
        pub(crate) mod compile;
        pub(crate) mod stringify;
    }
    pub(crate) mod directive_attributes {
        pub(crate) mod tokenize;
    }
    pub(crate) mod directive_container {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod directive_label {
        pub(crate) mod tokenize;
    }
    pub(crate) mod directive_leaf {
        pub(crate) mod tokenize;
    }
    pub(crate) mod directive_name {
        pub(crate) mod tokenize;
    }
    pub(crate) mod directive_text {
        pub(crate) mod tokenize;
    }
    pub(crate) mod emphasis {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod footnote_definition {
        pub(crate) mod compile;
        pub(crate) mod stringify;
    }
    pub(crate) mod footnote_reference {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod frontmatter {
        pub(crate) mod compile;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_autolink_literal {
        pub(crate) mod compile;
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_footnote_call {
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_footnote_definition {
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_potential_footnote_call {
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_strikethrough {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_table {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod gfm_task_list_item_check {
        pub(crate) mod compile;
        pub(crate) mod tokenize;
    }
    pub(crate) mod hard_break {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod hard_break_escape {
        pub(crate) mod tokenize;
    }
    pub(crate) mod heading {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod heading_atx {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod heading_setext {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod html {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod html_flow {
        pub(crate) mod tokenize;
    }
    pub(crate) mod html_text {
        pub(crate) mod tokenize;
    }
    pub(crate) mod image {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod image_reference {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod inline_code {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod inline_math {
        pub(crate) mod compile;
        pub(crate) mod stringify;
    }
    pub(crate) mod label {
        pub(crate) mod tokenize;
    }
    pub(crate) mod label_end {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
    pub(crate) mod label_start_image {
        pub(crate) mod tokenize;
    }
    pub(crate) mod label_start_link {
        pub(crate) mod tokenize;
    }
    pub(crate) mod line_ending {
        pub(crate) mod tokenize;
    }
    pub(crate) mod link {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod link_reference {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod list {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod list_item {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod resolve;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod markdown {
        pub(crate) mod parse;
        pub(crate) mod stringify;
    }
    pub(crate) mod math {
        pub(crate) mod compile;
        pub(crate) mod stringify;
    }
    pub(crate) mod math_flow {
        pub(crate) mod tokenize;
    }
    pub(crate) mod math_text {
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdast {
        pub(crate) mod bridge;
        pub(crate) mod convert;
    }
    pub(crate) mod mdx_expression {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdx_expression_flow {
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdx_expression_text {
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdx_jsx {
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdx_jsx_element {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod mdx_jsx_flow {
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdx_jsx_text {
        pub(crate) mod tokenize;
    }
    pub(crate) mod mdxjs_esm {
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod non_lazy_continuation {
        pub(crate) mod tokenize;
    }
    pub(crate) mod paragraph {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod resolve;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod root {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod space_or_tab {
        pub(crate) mod tokenize;
    }
    pub(crate) mod space_or_tab_eol {
        pub(crate) mod tokenize;
    }
    pub(crate) mod strong {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod table {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod table_cell {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod table_row {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod text {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod thematic_break {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod title {
        pub(crate) mod tokenize;
    }
    pub(crate) mod whitespace {
        pub(crate) mod resolve;
        pub(crate) mod tokenize;
    }
}
mod helpers {
    pub(crate) mod attribute;
    pub(crate) mod byte_set;
    pub(crate) mod character;
    pub(crate) mod content;
    pub(crate) mod decode;
    pub(crate) mod entities;
    pub(crate) mod html;
    pub(crate) mod identifier;
    pub(crate) mod indent;
    pub(crate) mod literal;
    pub(crate) mod math;
    pub(crate) mod position;
    pub(crate) mod reference;
    pub(crate) mod script;
    pub(crate) mod skip;
    pub(crate) mod slice;
    pub(crate) mod spread;
    pub(crate) mod table;
    pub(crate) mod unicode;
    pub(crate) mod uri;
}
mod plugins {
    pub(crate) mod definition_list {
        pub(crate) mod plugin;
    }
    pub(crate) mod directive {
        pub(crate) mod plugin;
    }
    pub(crate) mod frontmatter {
        pub(crate) mod plugin;
    }
    pub(crate) mod gemoji {
        pub(crate) mod actions {
            pub(crate) mod text {
                pub(crate) mod replace;
            }
        }
        pub(crate) mod names;
        pub(crate) mod plugin;
    }
    pub(crate) mod gfm {
        pub(crate) mod actions {
            pub(crate) mod text {
                pub(crate) mod autolink;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod math {
        pub(crate) mod plugin;
    }
    pub(crate) mod mdx {
        pub(crate) mod plugin;
    }
    pub(crate) mod parse {
        pub(crate) mod plugin;
    }
    pub(crate) mod stringify {
        pub(crate) mod plugin;
    }
    pub(crate) mod to_html {
        pub(crate) mod plugin;
    }
    pub(crate) mod to_prose {
        pub(crate) mod plugin;
    }
}

// models
pub use models::options::{
    DefinitionListSlot, DirectiveOptions, DirectiveSlot, FrontmatterOptions,
    FrontmatterSlot, GfmOptions, GfmSlot, MathOptions, MathSlot, Matter, MdxOptions,
    MdxSlot, Names, ParseOptions, StringifyOptions, ToHtmlOptions, ToProseOptions,
    UnsafePattern,
};

// actions
pub use actions::markdown::parse::parse_markdown;
pub use actions::markdown::stringify::stringify_markdown;
pub use actions::mdast::bridge::bridge_mdast;
pub use actions::mdast::convert::convert_mdast;

// plugins
pub use plugins::definition_list::plugin::{DefinitionList, definition_list};
pub use plugins::directive::plugin::{Directive, directive};
pub use plugins::frontmatter::plugin::{Frontmatter, frontmatter};
pub use plugins::gemoji::plugin::{Gemoji, gemoji};
pub use plugins::gfm::plugin::{Gfm, gfm};
pub use plugins::math::plugin::{Math, math};
pub use plugins::mdx::plugin::{Mdx, mdx};
pub use plugins::parse::plugin::{Parse, parse};
pub use plugins::stringify::plugin::{Stringify, stringify};
pub use plugins::to_html::plugin::{ToHtml, to_html};
pub use plugins::to_prose::plugin::{ToProse, to_prose};
