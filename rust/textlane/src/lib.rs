//! Textlane is a Rust-native text processing pipeline: a unified replacement whose plugins
//! run natively, with Node.js and Python bindings over the same crate.

mod actions {
    pub(crate) mod package {
        pub(crate) mod version;
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

pub use actions::package::version::read_version;
#[cfg(feature = "html")]
pub use textlane_html as html;
#[cfg(feature = "javascript")]
pub use textlane_javascript as javascript;
#[cfg(feature = "markdown")]
pub use textlane_markdown as markdown;
#[cfg(feature = "prose")]
pub use textlane_prose as prose;
