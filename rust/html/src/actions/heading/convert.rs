use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};
use crate::utils::converter::wrap::drop_surrounding_breaks;

/// `<h1>` to `<h6>`: a heading of that rank.
pub(crate) fn convert_heading<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let depth = element
        .tag_name
        .as_bytes()
        .get(1)
        .map_or(1, |digit| digit - b'0');
    let children = state.all(&element.children);
    let children = drop_surrounding_breaks(&state.store, children);
    Converted::One(
        state
            .store
            .add(Kind::Heading(depth), children, element.position),
    )
}
