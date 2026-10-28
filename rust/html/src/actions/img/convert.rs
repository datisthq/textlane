use textlane_core::hast::Element;

use crate::helpers::value::read_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<img>` and `<image>`: an image.
pub(crate) fn convert_img<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let source = read_value(element, "src");
    let url = state.resolve((!source.is_empty()).then_some(&*source));
    let title = state.read_optional(element, "title");
    let alt = state.read_optional(element, "alt").unwrap_or_default();
    let kind = Kind::Image { url, title, alt };
    Converted::One(state.store.add(kind, vec![], element.position))
}
