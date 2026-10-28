use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<del>`, `<s>` and `<strike>`: a delete.
pub(crate) fn convert_del<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    Converted::One(state.store.add(Kind::Delete, children, element.position))
}
