use textlane_core::hast::Element;

use crate::helpers::value::read_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<a>`: a link.
pub(crate) fn convert_a<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    let href = read_value(element, "href");
    let url = state.resolve((!href.is_empty()).then_some(&*href));
    let title = state.read_optional(element, "title");
    Converted::One(
        state
            .store
            .add(Kind::Link { url, title }, children, element.position),
    )
}
