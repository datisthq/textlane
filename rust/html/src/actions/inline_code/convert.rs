use textlane_core::hast::Element;

use crate::helpers::text::read_text;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<code>`, `<kbd>`, `<samp>`, `<tt>` and `<var>`: inline code of their text.
pub(crate) fn convert_inline_code<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let value = state.alloc.alloc_str(&read_text(element));
    Converted::One(
        state
            .store
            .add(Kind::InlineCode(value), vec![], element.position),
    )
}
