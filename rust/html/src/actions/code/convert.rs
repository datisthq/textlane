use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};

use crate::helpers::text::read_text;
use crate::helpers::value::find_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<pre>`, `<listing>`, `<plaintext>` and `<xmp>`: code of their text, in the language a
/// `<pre>`'s `<code>` names in a `language-*` class.
pub(crate) fn convert_code<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let mut lang = None;
    if element.tag_name == "pre" {
        let class_list = element.children.iter().find_map(|child| match child {
            Node::Element(child) if child.tag_name == "code" => {
                match find_value(child, "className") {
                    Some(PropertyValue::List(items)) => Some(items),
                    _ => None,
                }
            }
            _ => None,
        });
        lang = class_list.and_then(|items| {
            items.iter().find_map(|item| match item {
                PropertyItem::String(class) => class.strip_prefix("language-"),
                PropertyItem::Number(_) => None,
            })
        });
    }
    let text = read_text(element);
    let value = state.alloc.alloc_str(text.trim_end_matches(['\n', '\r']));
    let kind = Kind::Code {
        lang: lang.filter(|lang| !lang.is_empty()),
        value,
    };
    Converted::One(state.store.add(kind, vec![], element.position))
}
