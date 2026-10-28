use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::Converted;

/// `<wbr>`: a zero-width space.
pub(crate) fn convert_wbr<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    Converted::One(state.store.add_text("\u{200B}", element.position))
}
