use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<strong>` and `<b>`: strong.
pub(crate) fn convert_strong<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    Converted::One(state.store.add(Kind::Strong, children, element.position))
}
