use textlane_core::hast::Element;
use url::Url;

use crate::helpers::value::read_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::Converted;

/// `<base>`: the first one sets the URL later ones resolve against.
pub(crate) fn convert_base<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    if !state.base_found {
        // `new URL` throws on a relative base, which here leaves URLs as they are
        state.base = Url::parse(&read_value(element, "href")).ok();
        state.base_found = true;
    }
    Converted::Nothing
}
