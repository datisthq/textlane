use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<br>`: a hard break.
pub(crate) fn convert_br<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    Converted::One(state.store.add(Kind::Break, vec![], element.position))
}
