//! Textlane's core: the unist conventions and the mdast and hast nodes built on them, the file a
//! pipeline reads and reports diagnostics against, and the plugin contract every crate shares.

mod plugin;
mod models {
    pub(crate) mod error;
    pub(crate) mod file;
    pub(crate) mod hast;
    pub(crate) mod lines;
    pub(crate) mod mdast;
    pub(crate) mod position;
    pub(crate) mod tree;
}
#[cfg(feature = "json")]
mod actions {
    pub(crate) mod hast {
        pub(crate) mod deserialize;
        pub(crate) mod serialize;
    }
    pub(crate) mod mdast {
        pub(crate) mod deserialize;
        pub(crate) mod serialize;
    }
    pub(crate) mod position {
        pub(crate) mod deserialize;
        pub(crate) mod serialize;
    }
}

/// The mdast nodes: Markdown's syntax tree.
pub mod mdast {
    #[cfg(feature = "json")]
    pub use crate::actions::mdast::deserialize::deserialize_mdast;
    #[cfg(feature = "json")]
    pub use crate::actions::mdast::serialize::serialize_mdast;
    pub use crate::models::mdast::*;
}

/// The hast nodes: HTML's syntax tree.
pub mod hast {
    #[cfg(feature = "json")]
    pub use crate::actions::hast::deserialize::deserialize_hast;
    #[cfg(feature = "json")]
    pub use crate::actions::hast::serialize::serialize_hast;
    pub use crate::models::hast::*;
}

// models
pub use models::error::Error;
pub use models::file::{File, Message};
pub use models::lines::Lines;
pub use models::position::{Point, Position, UnistPoint, UnistPosition};
pub use models::tree::{Tree, TreeKind};
pub use plugin::{Attach, Compiler, Parser, Plugin, Slots, Transform};

/// The arena every node of one document is allocated in.
pub use bumpalo::Bump as Allocator;
/// A vector allocated in a document's arena.
pub type ArenaVec<'a, T> = bumpalo::collections::Vec<'a, T>;
/// A string allocated in a document's arena, while it is being built.
pub type ArenaString<'a> = bumpalo::collections::String<'a>;
