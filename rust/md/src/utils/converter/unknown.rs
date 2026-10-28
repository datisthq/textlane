use textlane_core::hast::{self, Property, PropertyItem, PropertyValue};
use textlane_core::{ArenaVec, Data, Position};

use crate::utils::converter::state::Converter;

/// A node without a handler of its own, as mdast-util-to-hast's default unknown handler
/// then `applyData` convert it: text, or a `div`, renamed and filled by `data`.
pub(crate) fn convert_unknown<'a>(
    converter: &Converter<'a, '_>,
    value: &'a str,
    data: Option<&'a Data<'a>>,
    position: Option<Position<'a>>,
) -> hast::Node<'a> {
    let alloc = converter.alloc;
    let h_name = read_field(data, "hName").and_then(read_string);
    let h_properties = read_field(data, "hProperties");
    let h_children = read_field(data, "hChildren");
    if h_properties.is_none() && h_children.is_none() {
        let text = converter.text(value);
        let Some(tag_name) = h_name else {
            return text;
        };
        let mut children = ArenaVec::with_capacity_in(1, alloc);
        children.push(text);
        return converter.element_with(tag_name, ArenaVec::new_in(alloc), children, None);
    }
    let mut properties = ArenaVec::new_in(alloc);
    if let Some(Data::Object(entries)) = h_properties {
        push_properties(converter, &mut properties, entries);
    }
    let children = match h_children {
        Some(Data::Array(items)) => convert_children(converter, items),
        _ => ArenaVec::new_in(alloc),
    };
    converter.element_with(h_name.unwrap_or("div"), properties, children, position)
}

fn convert_children<'a>(
    converter: &Converter<'a, '_>,
    items: &'a [Data<'a>],
) -> ArenaVec<'a, hast::Node<'a>> {
    let mut children = ArenaVec::with_capacity_in(items.len(), converter.alloc);
    children.extend(
        items
            .iter()
            .filter_map(|item| convert_node(converter, item)),
    );
    children
}

// `hChildren` is hast already, kept as JSON in `data`
fn convert_node<'a>(
    converter: &Converter<'a, '_>,
    data: &'a Data<'a>,
) -> Option<hast::Node<'a>> {
    let value = || {
        read_field(Some(data), "value")
            .and_then(read_string)
            .unwrap_or("")
    };
    match read_field(Some(data), "type").and_then(read_string)? {
        "text" => Some(converter.text(value())),
        "comment" => Some(hast::Node::Comment(hast::Comment {
            value: value(),
            position: None,
            data: None,
        })),
        "raw" => Some(hast::Node::Raw(hast::Raw {
            value: value(),
            position: None,
            data: None,
        })),
        "element" => {
            let mut properties = ArenaVec::new_in(converter.alloc);
            if let Some(Data::Object(entries)) = read_field(Some(data), "properties") {
                push_properties(converter, &mut properties, entries);
            }
            let children = match read_field(Some(data), "children") {
                Some(Data::Array(items)) => convert_children(converter, items),
                _ => ArenaVec::new_in(converter.alloc),
            };
            let tag_name = read_field(Some(data), "tagName").and_then(read_string)?;
            Some(converter.element_with(tag_name, properties, children, None))
        }
        _ => None,
    }
}

// A later property of the same name replaces an earlier one, as `Object.assign` does
fn push_properties<'a>(
    converter: &Converter<'a, '_>,
    properties: &mut ArenaVec<'a, Property<'a>>,
    entries: &'a [(&'a str, Data<'a>)],
) {
    for (name, value) in entries {
        if let Some(value) = convert_property(converter, value) {
            properties.retain(|property| property.name != *name);
            properties.push(Property { name, value });
        }
    }
}

fn convert_property<'a>(
    converter: &Converter<'a, '_>,
    data: &'a Data<'a>,
) -> Option<PropertyValue<'a>> {
    Some(match data {
        Data::Null => PropertyValue::Null,
        Data::Boolean(value) => PropertyValue::Boolean(*value),
        Data::Number(value) => PropertyValue::Number(*value),
        Data::String(value) => PropertyValue::String(value),
        Data::Array(items) => {
            let mut list = ArenaVec::with_capacity_in(items.len(), converter.alloc);
            list.extend(items.iter().filter_map(|item| match item {
                Data::String(value) => Some(PropertyItem::String(value)),
                Data::Number(value) => Some(PropertyItem::Number(*value)),
                _ => None,
            }));
            PropertyValue::List(list)
        }
        Data::Object(_) => return None,
    })
}

fn read_field<'a>(data: Option<&'a Data<'a>>, key: &str) -> Option<&'a Data<'a>> {
    match data? {
        Data::Object(entries) => entries
            .iter()
            .rev()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value),
        _ => None,
    }
}

fn read_string<'a>(data: &'a Data<'a>) -> Option<&'a str> {
    match data {
        Data::String(value) => Some(value),
        _ => None,
    }
}
