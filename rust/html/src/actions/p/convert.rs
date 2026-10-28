use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Id, Kind};
use crate::utils::converter::wrap::drop_surrounding_breaks;

/// `<p>` and `<summary>`: a paragraph, unless it is empty.
pub(crate) fn convert_p<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    build_paragraph(state, element, children)
}

/// The paragraph of a `<p>` whose children are already converted.
pub(crate) fn build_paragraph<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
    children: Vec<Id>,
) -> Converted {
    let children = drop_surrounding_breaks(&state.store, children);
    if children.is_empty() {
        return Converted::Nothing;
    }
    Converted::One(state.store.add(Kind::Paragraph, children, element.position))
}
