use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::state::{AttentionKind, Info, Parent, Serializer};

/// Write emphasis outside of phrasing.
pub(crate) fn stringify_emphasis(
    state: &mut Serializer,
    node: Parent,
    info: Info,
) -> Result<(), Error> {
    let kind = create_emphasis_attention(state.settings);
    state.serialize_attention(node, kind, info)
}

/// Emphasis is one marker, the configured one first.
pub(crate) fn create_emphasis_attention(settings: &Settings) -> AttentionKind {
    let other = if settings.emphasis == '*' { '_' } else { '*' };
    AttentionKind {
        construct: Construct::Emphasis,
        sequences: vec![settings.emphasis.to_string(), other.to_string()],
    }
}
