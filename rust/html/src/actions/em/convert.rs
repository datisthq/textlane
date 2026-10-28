use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<em>`, `<i>`, `<mark>` and `<u>`: emphasis.
pub(crate) fn convert_em<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    Converted::One(state.store.add(Kind::Emphasis, children, element.position))
}
