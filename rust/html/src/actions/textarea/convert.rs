use textlane_core::hast::Element;

use crate::helpers::text::read_text;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::Converted;

/// `<textarea>`: its text.
pub(crate) fn convert_textarea<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let value = state.alloc.alloc_str(&read_text(element));
    Converted::One(state.store.add_text(value, element.position))
}
