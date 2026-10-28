use textlane_core::hast::Element;

use crate::helpers::value::{find_value, is_truthy, parse_int, write_value};
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<ol>`, `<ul>` and `<dir>`: a list, its content in list items.
pub(crate) fn convert_list<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let ordered = element.tag_name == "ol";
    let children = state.all(&element.children);
    let children = state.wrap_specific_content(children, Kind::list_item());
    // A start that is not a number is `NaN`, which JSON writes as `null`; mdast holds no
    // negative start, so one saturates at zero
    let start = match find_value(element, "start") {
        _ if !ordered => None,
        Some(start) if is_truthy(Some(start)) => {
            parse_int(&write_value(start)).map(|start| start as u32)
        }
        _ => Some(1),
    };
    let spread = state.store.is_spread(&children);
    let kind = Kind::List {
        ordered,
        start,
        spread,
    };
    Converted::One(state.store.add(kind, children, element.position))
}
