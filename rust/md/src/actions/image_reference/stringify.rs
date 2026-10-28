use textlane_core::mdast::{ImageReference, ReferenceType};

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::read_association;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::Serializer;

/// Write an image reference, `![alt][label]`.
pub(crate) fn stringify_image_reference(
    state: &mut Serializer,
    reference: &ImageReference,
) {
    state.enter(Construct::ImageReference);
    state.enter(Construct::Label);
    let mut value = "![".to_string();
    let alt = state.safe(
        reference.alt.unwrap_or_default(),
        &SafeConfig {
            before: &value,
            after: "]",
            ..SafeConfig::default()
        },
    );
    value.push_str(&alt);
    value.push_str("][");
    state.exit();
    let label = read_association(reference.identifier, reference.label);
    let association = write_association(state, &value, &label);
    state.exit();
    close_reference(&mut value, reference.reference_type, &alt, &association);
    state.write(&value);
}

/// The escaped label of a reference, written outside phrasing.
pub(crate) fn write_association(
    state: &mut Serializer,
    before: &str,
    label: &str,
) -> String {
    state.without_phrasing(|state| {
        state.enter(Construct::Reference);
        let value = state.safe(
            label,
            &SafeConfig {
                before,
                after: "]",
                ..SafeConfig::default()
            },
        );
        state.exit();
        value
    })
}

/// End `value`, which ends in `][`, as a full, collapsed or shortcut reference; full
/// whenever the text would not find the same definition.
pub(crate) fn close_reference(
    value: &mut String,
    kind: ReferenceType,
    text: &str,
    association: &str,
) {
    if kind == ReferenceType::Full || text.is_empty() || text != association {
        value.push_str(association);
        value.push(']');
    } else if kind == ReferenceType::Shortcut {
        value.pop();
    } else {
        value.push(']');
    }
}
