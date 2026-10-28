//! Textlane is a Rust-native text processing pipeline: a unified replacement whose plugins
//! run natively, with Node.js and Python bindings over the same crate.

mod models {
    pub(crate) mod pipeline;
    pub(crate) mod processor;
}
mod actions {
    #[cfg(feature = "testing")]
    pub(crate) mod case {
        pub(crate) mod run;
    }
    pub(crate) mod package {
        pub(crate) mod version;
    }
    pub(crate) mod plugin {
        pub(crate) mod find;
    }
}
#[cfg(feature = "testing")]
mod utils {
    pub(crate) mod testing {
        pub(crate) mod plugins;
    }
}

#[cfg(any(feature = "node", feature = "python"))]
mod bindings {
    // napi's macros expand to unsafe FFI glue and register exports through a constructor,
    // so rustc sees neither the unsafe as ours nor the exports as used
    #[cfg(feature = "node")]
    #[allow(unsafe_code, dead_code)]
    pub(crate) mod node;
    #[cfg(feature = "python")]
    pub(crate) mod python;
}

#[cfg(feature = "testing")]
pub use actions::case::run::run_case;
pub use actions::package::version::read_version;
pub use models::pipeline::{Api, Pipeline};
pub use models::processor::{FrozenProcessor, Output, Processor};
pub use textlane_core::*;
#[cfg(feature = "html")]
pub use textlane_html as html;
#[cfg(feature = "md")]
pub use textlane_md as md;
#[cfg(feature = "mdx")]
pub use textlane_mdx as mdx;
#[cfg(feature = "prose")]
pub use textlane_prose as prose;
