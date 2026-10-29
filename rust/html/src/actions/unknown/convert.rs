use textlane_core::Data;
use textlane_core::hast::Unknown;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::Converted;

/// A node of an unknown type: text when it has a string `value`, else its children.
pub(crate) fn convert_unknown<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    node: &'n Unknown<'a>,
) -> Converted {
    let value = node.fields.iter().find_map(|(name, value)| match value {
        Data::String(value) if *name == "value" => Some(*value),
        _ => None,
    });
    if let Some(value) = value {
        return Converted::One(state.store.add_text(value, node.position));
    }
    match &node.children {
        Some(children) => Converted::Many(state.all(children)),
        None => Converted::Nothing,
    }
}
