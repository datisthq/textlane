//! Textlane's Markdown plugins: parsing, syntax extensions, mdast transforms, serializing and
//! the bridge to HTML.

mod plugin;
mod models {
    pub(crate) mod mdx;
    pub(crate) mod options;
}
mod utils {
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
    pub(crate) mod onepass {
        pub(crate) mod block;
        pub(crate) mod build;
        pub(crate) mod directive;
        pub(crate) mod inline;
        pub(crate) mod literal;
        pub(crate) mod mdx;
        pub(crate) mod scan;
    }
}
mod actions {
    pub(crate) mod blockquote {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod code {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod def_list {
        pub(crate) mod convert;
        pub(crate) mod stringify;
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
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod delete {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod directive {
        pub(crate) mod stringify;
    }
    pub(crate) mod emphasis {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod footnote_definition {
        pub(crate) mod stringify;
    }
    pub(crate) mod footnote_reference {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod frontmatter {
        pub(crate) mod stringify;
    }
    pub(crate) mod hard_break {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod heading {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod html {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod image {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod image_reference {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod inline_code {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod inline_math {
        pub(crate) mod stringify;
    }
    pub(crate) mod link {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod link_reference {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod list {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod list_item {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod markdown {
        pub(crate) mod parse;
        pub(crate) mod stringify;
    }
    pub(crate) mod math {
        pub(crate) mod stringify;
    }
    pub(crate) mod mdast {
        pub(crate) mod bridge;
        pub(crate) mod convert;
    }
    pub(crate) mod mdx_expression {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod mdx_jsx_element {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod mdxjs_esm {
        pub(crate) mod stringify;
    }
    pub(crate) mod paragraph {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod root {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod strong {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod table {
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
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
    pub(crate) mod thematic_break {
        pub(crate) mod convert;
        pub(crate) mod stringify;
    }
}
mod helpers {
    pub(crate) mod attribute;
    pub(crate) mod character;
    pub(crate) mod decode;
    pub(crate) mod entities;
    pub(crate) mod html;
    pub(crate) mod identifier;
    pub(crate) mod literal;
    pub(crate) mod math;
    pub(crate) mod position;
    pub(crate) mod reference;
    pub(crate) mod script;
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
