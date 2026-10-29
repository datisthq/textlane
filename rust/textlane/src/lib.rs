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
    pub(crate) mod file {
        pub(crate) mod write;
    }
    pub(crate) mod package {
        pub(crate) mod version;
    }
    pub(crate) mod plugin {
        #[cfg(feature = "schema")]
        pub(crate) mod describe;
        pub(crate) mod find;
        pub(crate) mod list;
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
pub use actions::file::write::write_file;
pub use actions::package::version::read_version;
#[cfg(feature = "schema")]
pub use actions::plugin::describe::describe_plugins;
pub use actions::plugin::list::list_plugins;
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
