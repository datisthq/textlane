use textlane_core::hast::Element;

use crate::helpers::selected::find_selected_options;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::Converted;

/// `<select>`: the text of its selected options.
pub(crate) fn convert_select<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let values = find_selected_options(element, element);
    if values.is_empty() {
        return Converted::Nothing;
    }
    let texts: Vec<String> = values
        .into_iter()
        .map(|(value, label)| match label {
            Some(label) => format!("{label} ({value})"),
            None => value,
        })
        .collect();
    let value = state.alloc.alloc_str(&texts.join(", "));
    Converted::One(state.store.add_text(value, element.position))
}
