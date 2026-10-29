//! Textlane's MDX compiler: hast into a JS tree read through oxc, the MDX transforms and
//! code generation as astring prints it.

mod models {
    pub(crate) mod estree;
    pub(crate) mod name;
    pub(crate) mod options;
    pub(crate) mod scope;
    pub(crate) mod script;
}
mod utils {
    pub(crate) mod printer {
        pub(crate) mod expression;
        pub(crate) mod jsx;
        pub(crate) mod precedence;
        pub(crate) mod state;
        pub(crate) mod statement;
    }
}
mod actions {
    pub(crate) mod comment {
        pub(crate) mod attach;
    }
    pub(crate) mod expression {
        pub(crate) mod parse;
        pub(crate) mod read;
    }
    pub(crate) mod hast {
        pub(crate) mod compile;
        pub(crate) mod convert;
    }
    pub(crate) mod module {
        pub(crate) mod parse;
    }
    pub(crate) mod program {
        pub(crate) mod build;
        pub(crate) mod rewrite;
        pub(crate) mod wrap;
    }
    pub(crate) mod script {
        pub(crate) mod parse;
        pub(crate) mod read;
    }
}
mod helpers {
    pub(crate) mod diagnostic;
    pub(crate) mod entity;
    pub(crate) mod entity_table;
    pub(crate) mod identifier;
    pub(crate) mod json;
    pub(crate) mod place;
    pub(crate) mod property;
    pub(crate) mod property_table;
    pub(crate) mod style;
    pub(crate) mod walk;
}
mod plugins {
    pub(crate) mod compile {
        pub(crate) mod plugin;
    }
}

// models
pub use models::options::CompileOptions;
pub use models::script::OxcScriptParser;

// actions
pub use actions::hast::compile::compile_hast;

// plugins
pub use plugins::compile::plugin::{Compile, compile};
