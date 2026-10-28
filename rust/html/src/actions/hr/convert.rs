use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<hr>`: a thematic break.
pub(crate) fn convert_hr<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    Converted::One(
        state
            .store
            .add(Kind::ThematicBreak, vec![], element.position),
    )
}
