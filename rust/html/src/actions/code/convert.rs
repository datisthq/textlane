use std::collections::BTreeMap;

use textlane_core::Data;
use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};

use crate::actions::attribute::read::read_attribute;

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
    let meta = read_meta(element, &state.options.code_meta);
    let kind = Kind::Code {
        lang: lang.filter(|lang| !lang.is_empty()),
        meta: meta.map(|meta| &*state.alloc.alloc_str(&meta)),
        value,
    };
    Converted::One(state.store.add(kind, vec![], element.position))
}

// The `<code>`'s meta as mdast-util-to-hast keeps it in `data.meta`, then each attribute
// `codeMeta` names, as `key="value"`
fn read_meta(pre: &Element<'_>, names: &BTreeMap<String, String>) -> Option<String> {
    let code = pre.children.iter().find_map(|child| match child {
        Node::Element(code) if code.tag_name == "code" => Some(code),
        _ => None,
    });
    let mut parts: Vec<String> = vec![];
    if let Some(Data::Object(entries)) = code.and_then(|code| code.data)
        && let Some((_, Data::String(meta))) =
            entries.iter().find(|(key, _)| *key == "meta")
    {
        parts.push((*meta).to_owned());
    }
    for (name, key) in names {
        let value = code
            .and_then(|code| read_attribute(code, name))
            .or_else(|| read_attribute(pre, name));
        if let Some(value) = value {
            let quote = if value.contains('"') { '\'' } else { '"' };
            parts.push(format!("{key}={quote}{value}{quote}"));
        }
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}
