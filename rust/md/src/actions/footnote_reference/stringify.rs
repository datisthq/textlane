use textlane_core::mdast::FootnoteReference;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::read_association;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::Serializer;

/// Write a call to a GFM footnote, `[^a]`.
pub(crate) fn stringify_footnote_reference(
    state: &mut Serializer,
    reference: &FootnoteReference,
) {
    let mut value = "[^".to_string();
    state.enter(Construct::Reference);
    let label = read_association(reference.identifier, reference.label);
    let safe = state.safe(
        &label,
        &SafeConfig {
            before: &value,
            after: "]",
            ..SafeConfig::default()
        },
    );
    state.exit();
    value.push_str(&safe);
    value.push(']');
    state.write(&value);
}
