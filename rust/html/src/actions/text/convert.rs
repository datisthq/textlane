use textlane_core::hast::Text;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::Converted;

/// Text.
pub(crate) fn convert_text<'a>(
    state: &mut Converter<'a, '_, '_>,
    text: &Text<'a>,
) -> Converted {
    Converted::One(state.store.add_text(text.value, text.position))
}
