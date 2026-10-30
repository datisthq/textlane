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
    pub(crate) mod writer {
        pub(crate) mod escape;
        pub(crate) mod state;
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
    pub(crate) mod yaml {
        pub(crate) mod compose;
        pub(crate) mod convert;
        pub(crate) mod document;
        pub(crate) mod number;
        pub(crate) mod parse;
        pub(crate) mod scalar;
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
        pub(crate) mod write;
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
    pub(crate) mod command;
    pub(crate) mod command_table;
    pub(crate) mod decode;
    pub(crate) mod entities;
    pub(crate) mod html;
    pub(crate) mod identifier;
    pub(crate) mod lines;
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
    pub(crate) mod component {
        pub(crate) mod actions {
            pub(crate) mod alert {
                pub(crate) mod convert;
            }
            pub(crate) mod children {
                pub(crate) mod group;
            }
            pub(crate) mod code {
                pub(crate) mod convert;
            }
            pub(crate) mod directive {
                pub(crate) mod convert;
                pub(crate) mod restore;
            }
            pub(crate) mod element {
                pub(crate) mod create;
            }
            pub(crate) mod meta {
                pub(crate) mod read;
            }
            pub(crate) mod node {
                pub(crate) mod convert;
            }
            pub(crate) mod text {
                pub(crate) mod read;
            }
            pub(crate) mod tree {
                pub(crate) mod read;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod definition_list {
        pub(crate) mod plugin;
    }
    pub(crate) mod heading_id {
        pub(crate) mod actions {
            pub(crate) mod data {
                pub(crate) mod copy;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod directive {
        pub(crate) mod plugin;
    }
    pub(crate) mod frontmatter {
        pub(crate) mod actions {
            pub(crate) mod matter {
                pub(crate) mod read;
            }
        }
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
    pub(crate) mod package_command {
        pub(crate) mod actions {
            pub(crate) mod args {
                pub(crate) mod parse;
            }
            pub(crate) mod bun {
                pub(crate) mod convert;
            }
            pub(crate) mod cargo {
                pub(crate) mod convert;
            }
            pub(crate) mod code {
                pub(crate) mod convert;
            }
            pub(crate) mod deno {
                pub(crate) mod convert;
            }
            pub(crate) mod npm {
                pub(crate) mod convert;
            }
            pub(crate) mod nub {
                pub(crate) mod convert;
            }
            pub(crate) mod pip {
                pub(crate) mod convert;
            }
            pub(crate) mod pnpm {
                pub(crate) mod convert;
            }
            pub(crate) mod yarn {
                pub(crate) mod convert;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod parse {
        pub(crate) mod plugin;
    }
    pub(crate) mod path_rewrite {
        pub(crate) mod actions {
            pub(crate) mod url {
                pub(crate) mod rewrite;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod stringify {
        pub(crate) mod plugin;
    }
    pub(crate) mod strip {
        pub(crate) mod actions {
            pub(crate) mod list {
                pub(crate) mod strip;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod toc {
        pub(crate) mod actions {
            pub(crate) mod contents {
                pub(crate) mod build;
            }
            pub(crate) mod expression {
                pub(crate) mod compile;
            }
            pub(crate) mod heading {
                pub(crate) mod search;
            }
            pub(crate) mod node {
                pub(crate) mod copy;
            }
            pub(crate) mod parent {
                pub(crate) mod check;
            }
            pub(crate) mod text {
                pub(crate) mod collect;
            }
        }
        pub(crate) mod plugin;
    }
    pub(crate) mod to_html {
        pub(crate) mod plugin;
    }
    pub(crate) mod to_natural_language {
        pub(crate) mod plugin;
    }
}

// models
pub use models::options::{
    AlertRule, AttributeRule, CargoManager, CodeGroupRule, CodeRule, Component,
    DefinitionListSlot, Directive, DirectiveKind, DirectiveRule, DirectiveSlot,
    Ecosystems, Frontmatter, FrontmatterSlot, Gfm, GfmSlot, GroupRule, HeadingId,
    MarkdownLinks, Math, MathSlot, Matter, Mdx, MdxSlot, Names, NpmManager,
    PackageCommand, ParentCheck, ParentTest, Parse, PathRewrite, PipManager, Stringify,
    Strip, ToHtml, ToNaturalLanguage, Toc, UnknownDirectives, UnsafePattern, UrlNode,
};

// actions
pub use actions::markdown::parse::parse_markdown;
pub use actions::markdown::stringify::stringify_markdown;
pub use actions::mdast::bridge::bridge_mdast;
pub use actions::mdast::convert::convert_mdast;
pub use actions::mdast::write::write_html;

// plugins
pub use plugins::definition_list::plugin::DefinitionList;
pub use plugins::gemoji::plugin::Gemoji;
