use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<blockquote>`: a block quote of flow content.
pub(crate) fn convert_blockquote<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    let children = state.wrap_flow(children);
    Converted::One(
        state
            .store
            .add(Kind::Blockquote, children, element.position),
    )
}
