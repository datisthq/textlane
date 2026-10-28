use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::state::{AttentionKind, Info, Parent, Serializer};

/// Write strong outside of phrasing.
pub(crate) fn stringify_strong(
    state: &mut Serializer,
    node: Parent,
    info: Info,
) -> Result<(), Error> {
    let kind = create_strong_attention(state.settings);
    state.serialize_attention(node, kind, info)
}

/// Strong is two markers, the configured one first.
pub(crate) fn create_strong_attention(settings: &Settings) -> AttentionKind {
    let other = if settings.strong == '*' { '_' } else { '*' };
    AttentionKind {
        construct: Construct::Strong,
        sequences: vec![
            settings.strong.to_string().repeat(2),
            other.to_string().repeat(2),
        ],
    }
}
