//! Textlane's HTML plugins: parsing, hast transforms, sanitizing and serializing.

mod models {
    pub(crate) mod options;
}
mod actions {
    pub(crate) mod hast {
        pub(crate) mod stringify;
    }
}
mod plugins {
    pub(crate) mod stringify {
        pub(crate) mod plugin;
    }
}

// models
pub use models::options::StringifyOptions;

// actions
pub use actions::hast::stringify::stringify_hast;

// plugins
pub use plugins::stringify::plugin::{Stringify, stringify};
