use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{AttentionKind, Info, Parent, Serializer};

/// Write a delete outside of phrasing.
pub(crate) fn stringify_delete(
    state: &mut Serializer,
    node: Parent,
    info: Info,
) -> Result<(), Error> {
    state.serialize_attention(node, create_delete_attention(), info)
}

/// A delete is two tildes, or one.
pub(crate) fn create_delete_attention() -> AttentionKind {
    AttentionKind {
        construct: Construct::Strikethrough,
        sequences: vec!["~~".to_string(), "~".to_string()],
    }
}
