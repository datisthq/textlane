//! Textlane's HTML plugins: parsing, hast transforms, sanitizing and serializing.

mod models {
    pub(crate) mod namespace;
    pub(crate) mod options;
    pub(crate) mod parse_error;
    pub(crate) mod token;
}
mod utils {
    pub(crate) mod builder {
        pub(crate) mod convert;
        pub(crate) mod dom;
        pub(crate) mod formatting;
        pub(crate) mod modes;
        pub(crate) mod parser;
    }
    pub(crate) mod tokenizer {
        pub(crate) mod entity;
        pub(crate) mod machine;
        pub(crate) mod preprocess;
    }
}
mod actions {
    pub(crate) mod hast {
        pub(crate) mod stringify;
    }
    pub(crate) mod html {
        pub(crate) mod parse;
    }
}
mod helpers {
    pub(crate) mod doctype;
    pub(crate) mod entities;
    pub(crate) mod foreign;
    pub(crate) mod property;
    pub(crate) mod reason;
    pub(crate) mod schema;
    pub(crate) mod tag;
}
mod plugins {
    pub(crate) mod parse {
        pub(crate) mod plugin;
    }
    pub(crate) mod stringify {
        pub(crate) mod plugin;
    }
}

// models
pub use models::options::{ParseOptions, StringifyOptions};

// actions
pub use actions::hast::stringify::stringify_hast;
pub use actions::html::parse::parse_html;

// plugins
pub use plugins::parse::plugin::{Parse, parse};
pub use plugins::stringify::plugin::{Stringify, stringify};
