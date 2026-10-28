use textlane_core::hast::Element;

use crate::helpers::value::read_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<iframe>`: a link to its source, named by its title, when it has both.
pub(crate) fn convert_iframe<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let source = read_value(element, "src");
    let title = read_value(element, "title");
    if source.is_empty() || title.is_empty() {
        return Converted::Nothing;
    }
    let url = state.resolve(Some(&source));
    let text = state.intern(title);
    let text = state.store.add_text(text, None);
    let kind = Kind::Link { url, title: None };
    Converted::One(state.store.add(kind, vec![text], element.position))
}
