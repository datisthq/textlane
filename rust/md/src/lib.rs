//! Textlane's Markdown plugins: parsing, syntax extensions, mdast transforms, serializing and
//! the bridge to HTML.

mod plugin;
mod models {
    mod code;
    mod construct;
    mod event;
    mod options;
    mod token;
}
mod utils {
    mod compiler {
        mod handlers;
        mod stack;
    }
    mod converter {
        mod footnote;
        mod state;
    }
    mod serializer {
        mod join;
        mod safe;
        mod state;
        mod unsafe_patterns;
    }
    mod tokenizer {
        mod content;
        mod driver;
        mod preprocess;
        mod registry;
        mod subtokenize;
    }
}
mod actions {
    mod attention {
        mod resolve;
        mod tokenize;
    }
    mod autolink {
        mod tokenize;
    }
    mod blank_line {
        mod tokenize;
    }
    mod block_quote {
        mod tokenize;
    }
    mod blockquote {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod character_escape {
        mod tokenize;
    }
    mod character_reference {
        mod tokenize;
    }
    mod code {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod code_fenced {
        mod tokenize;
    }
    mod code_indented {
        mod tokenize;
    }
    mod code_text {
        mod resolve;
        mod tokenize;
    }
    mod definition {
        mod compile;
        mod convert;
        mod stringify;
        mod tokenize;
    }
    mod destination {
        mod tokenize;
    }
    mod emphasis {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod hard_break {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod hard_break_escape {
        mod tokenize;
    }
    mod heading {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod heading_atx {
        mod resolve;
        mod tokenize;
    }
    mod heading_setext {
        mod resolve;
        mod tokenize;
    }
    mod html {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod html_flow {
        mod tokenize;
    }
    mod html_text {
        mod tokenize;
    }
    mod image {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod image_reference {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod inline_code {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod label {
        mod tokenize;
    }
    mod label_end {
        mod resolve;
        mod tokenize;
    }
    mod label_start_image {
        mod tokenize;
    }
    mod label_start_link {
        mod tokenize;
    }
    mod line_ending {
        mod tokenize;
    }
    mod link {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod link_reference {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod list {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod list_item {
        mod compile;
        mod convert;
        mod stringify;
        mod tokenize;
    }
    mod markdown {
        mod parse;
        mod stringify;
    }
    mod mdast {
        mod convert;
    }
    mod paragraph {
        mod compile;
        mod convert;
        mod resolve;
        mod stringify;
        mod tokenize;
    }
    mod root {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod space_or_tab {
        mod tokenize;
    }
    mod strong {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod text {
        mod compile;
        mod convert;
        mod stringify;
    }
    mod thematic_break {
        mod compile;
        mod convert;
        mod stringify;
        mod tokenize;
    }
    mod title {
        mod tokenize;
    }
    mod whitespace {
        mod tokenize;
    }
}
mod helpers {
    mod character;
    mod entities;
    mod identifier;
    mod position;
    mod reference;
    mod uri;
}
mod plugins {
    mod definition_list {
        mod plugin;
    }
    mod directive {
        mod plugin;
    }
    mod frontmatter {
        mod plugin;
    }
    mod gemoji {
        mod plugin;
    }
    mod gfm {
        mod plugin;
    }
    mod math {
        mod plugin;
    }
    mod mdx {
        mod plugin;
    }
    mod parse {
        mod plugin;
    }
    mod stringify {
        mod plugin;
    }
    mod to_html {
        mod plugin;
    }
}
