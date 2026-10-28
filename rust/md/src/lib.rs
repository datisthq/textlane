//! Textlane's Markdown plugins: parsing, syntax extensions, mdast transforms, serializing and
//! the bridge to HTML.

mod plugin;
mod models {
    pub(crate) mod code;
    pub(crate) mod construct;
    pub(crate) mod event;
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
    }
    pub(crate) mod serializer {
        pub(crate) mod join;
        pub(crate) mod safe;
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
    pub(crate) mod bom {
        pub(crate) mod tokenize;
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
    pub(crate) mod definition {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
        pub(crate) mod tokenize;
    }
    pub(crate) mod destination {
        pub(crate) mod tokenize;
    }
    pub(crate) mod emphasis {
        pub(crate) mod compile;
        pub(crate) mod convert;
        pub(crate) mod stringify;
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
    pub(crate) mod mdast {
        pub(crate) mod convert;
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
    pub(crate) mod character;
    pub(crate) mod decode;
    pub(crate) mod entities;
    pub(crate) mod html;
    pub(crate) mod identifier;
    pub(crate) mod position;
    pub(crate) mod reference;
    pub(crate) mod skip;
    pub(crate) mod slice;
    pub(crate) mod spread;
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
        pub(crate) mod plugin;
    }
    pub(crate) mod gfm {
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
}

// models
pub use models::options::{ParseOptions, ToHtmlOptions};

// actions
pub use actions::markdown::parse::parse_markdown;
pub use actions::mdast::convert::convert_mdast;

// plugins
pub use plugins::parse::plugin::{Parse, parse};
pub use plugins::to_html::plugin::{ToHtml, to_html};
