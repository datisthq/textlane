//! Textlane's HTML plugins: parsing, hast transforms, sanitizing and serializing.

mod models {
    #[cfg(feature = "highlight")]
    pub(crate) mod highlight;
    pub(crate) mod namespace;
    pub(crate) mod options;
    pub(crate) mod parse_error;
    pub(crate) mod token;
}
mod utils {
    #[cfg(feature = "highlight")]
    pub(crate) mod ansi {
        pub(crate) mod palette;
        pub(crate) mod parse;
    }
    pub(crate) mod builder {
        pub(crate) mod convert;
        pub(crate) mod dom;
        pub(crate) mod feed;
        pub(crate) mod formatting;
        pub(crate) mod modes;
        pub(crate) mod parser;
    }
    pub(crate) mod converter {
        pub(crate) mod state;
        pub(crate) mod store;
        pub(crate) mod wrap;
    }
    #[cfg(feature = "highlight")]
    pub(crate) mod notation {
        pub(crate) mod apply;
        pub(crate) mod parse;
    }
    pub(crate) mod tokenizer {
        pub(crate) mod entity;
        pub(crate) mod machine;
        pub(crate) mod preprocess;
    }
}
mod actions {
    pub(crate) mod a {
        pub(crate) mod convert;
    }
    #[cfg(feature = "highlight")]
    pub(crate) mod ansi {
        pub(crate) mod tokenize;
    }
    pub(crate) mod base {
        pub(crate) mod convert;
    }
    pub(crate) mod blockquote {
        pub(crate) mod convert;
    }
    pub(crate) mod br {
        pub(crate) mod convert;
    }
    pub(crate) mod code {
        pub(crate) mod convert;
        #[cfg(feature = "highlight")]
        pub(crate) mod decorate;
        #[cfg(feature = "highlight")]
        pub(crate) mod highlight;
        #[cfg(feature = "highlight")]
        pub(crate) mod tokenize;
        #[cfg(feature = "highlight")]
        pub(crate) mod transform;
    }
    pub(crate) mod comment {
        pub(crate) mod convert;
    }
    pub(crate) mod del {
        pub(crate) mod convert;
    }
    pub(crate) mod dl {
        pub(crate) mod convert;
    }
    pub(crate) mod em {
        pub(crate) mod convert;
    }
    pub(crate) mod hast {
        pub(crate) mod bridge;
        pub(crate) mod convert;
        pub(crate) mod external;
        pub(crate) mod format;
        #[cfg(feature = "highlight")]
        pub(crate) mod highlight;
        pub(crate) mod minify;
        pub(crate) mod raw;
        pub(crate) mod sanitize;
        pub(crate) mod slug;
        pub(crate) mod stringify;
        pub(crate) mod toc;
        #[cfg(feature = "katex")]
        pub(crate) mod typeset;
        pub(crate) mod unwrap;
    }
    pub(crate) mod heading {
        pub(crate) mod convert;
    }
    pub(crate) mod hr {
        pub(crate) mod convert;
    }
    pub(crate) mod html {
        pub(crate) mod parse;
    }
    pub(crate) mod iframe {
        pub(crate) mod convert;
    }
    pub(crate) mod img {
        pub(crate) mod convert;
    }
    pub(crate) mod inline_code {
        pub(crate) mod convert;
    }
    pub(crate) mod input {
        pub(crate) mod convert;
    }
    pub(crate) mod li {
        pub(crate) mod convert;
    }
    pub(crate) mod list {
        pub(crate) mod convert;
    }
    #[cfg(feature = "katex")]
    pub(crate) mod math {
        pub(crate) mod typeset;
    }
    pub(crate) mod media {
        pub(crate) mod convert;
    }
    pub(crate) mod p {
        pub(crate) mod convert;
    }
    pub(crate) mod q {
        pub(crate) mod convert;
    }
    pub(crate) mod root {
        pub(crate) mod convert;
    }
    pub(crate) mod select {
        pub(crate) mod convert;
    }
    pub(crate) mod strong {
        pub(crate) mod convert;
    }
    pub(crate) mod table {
        pub(crate) mod convert;
    }
    pub(crate) mod table_cell {
        pub(crate) mod convert;
    }
    pub(crate) mod table_row {
        pub(crate) mod convert;
    }
    pub(crate) mod text {
        pub(crate) mod convert;
    }
    pub(crate) mod textarea {
        pub(crate) mod convert;
    }
    #[cfg(feature = "highlight")]
    pub(crate) mod theme {
        pub(crate) mod resolve;
    }
    pub(crate) mod unknown {
        pub(crate) mod convert;
    }
    pub(crate) mod wbr {
        pub(crate) mod convert;
    }
}
mod helpers {
    pub(crate) mod doctype;
    pub(crate) mod entities;
    pub(crate) mod foreign;
    pub(crate) mod omission;
    pub(crate) mod phrasing;
    pub(crate) mod property;
    pub(crate) mod reason;
    #[cfg(feature = "highlight")]
    pub(crate) mod registry;
    pub(crate) mod schema;
    pub(crate) mod selected;
    pub(crate) mod tag;
    pub(crate) mod text;
    #[cfg(feature = "highlight")]
    pub(crate) mod theme_table;
    pub(crate) mod value;
}
mod plugins {
    pub(crate) mod external_links {
        pub(crate) mod plugin;
    }
    pub(crate) mod extract_toc {
        pub(crate) mod plugin;
    }
    pub(crate) mod format {
        pub(crate) mod plugin;
    }
    #[cfg(feature = "highlight")]
    pub(crate) mod highlight {
        pub(crate) mod plugin;
    }
    #[cfg(feature = "katex")]
    pub(crate) mod katex {
        pub(crate) mod plugin;
    }
    pub(crate) mod parse {
        pub(crate) mod plugin;
    }
    pub(crate) mod raw {
        pub(crate) mod plugin;
    }
    pub(crate) mod sanitize {
        pub(crate) mod plugin;
    }
    pub(crate) mod slug {
        pub(crate) mod plugin;
    }
    pub(crate) mod stringify {
        pub(crate) mod plugin;
    }
    pub(crate) mod to_markdown {
        pub(crate) mod plugin;
    }
    pub(crate) mod to_natural_language {
        pub(crate) mod plugin;
    }
    pub(crate) mod unwrap_images {
        pub(crate) mod plugin;
    }
}

// models
pub use models::options::{
    AttributeDefinition, Classes, ColorReplacement, ConfiguredTransformer, ExternalLinks,
    Format, Highlight, HighlightTransformer, Indent, InlineCode, Katex, KatexOutput,
    KatexStrict, KatexStrictMode, Parse, Primitive, Raw, Sanitize, Slug, Stringify,
    ToMarkdown, Tokens, TransformerName,
};

// actions
pub use actions::hast::bridge::bridge_hast;
pub use actions::hast::convert::convert_hast;
pub use actions::hast::external::mark_external_links;
pub use actions::hast::format::format_hast;
#[cfg(feature = "highlight")]
pub use actions::hast::highlight::highlight_hast;
pub use actions::hast::raw::raw_hast;
pub use actions::hast::sanitize::sanitize_hast;
pub use actions::hast::slug::slug_hast;
pub use actions::hast::stringify::stringify_hast;
pub use actions::hast::toc::extract_toc;
#[cfg(feature = "katex")]
pub use actions::hast::typeset::typeset_hast;
pub use actions::hast::unwrap::unwrap_images;
pub use actions::html::parse::parse_html;

// plugins
pub use plugins::extract_toc::plugin::ExtractToc;
pub use plugins::to_natural_language::plugin::ToNaturalLanguage;
pub use plugins::unwrap_images::plugin::UnwrapImages;
