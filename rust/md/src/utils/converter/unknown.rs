use textlane_core::hast::{self, Property, PropertyItem, PropertyValue};
use textlane_core::mdast;
use textlane_core::{ArenaVec, Data};

use crate::utils::converter::state::Converter;
use crate::utils::serializer::state::read_node_type;

/// A node without a handler: copied when its type is in `passThrough`, else, as
/// mdast-util-to-hast's default unknown handler does, text of its value or a `div` of its
/// children, renamed and filled by `data`.
pub(crate) fn convert_unknown<'a>(
    converter: &Converter<'a, '_>,
    node: &mdast::Node<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let kind = read_node_type(node);
    if converter
        .options
        .pass_through
        .iter()
        .any(|name| name == kind)
    {
        return pass_through(converter, node, kind, children);
    }
    let data = node.data();
    let position = node.position().copied();
    let styled = read_field(data, "hProperties").is_some()
        || read_field(data, "hChildren").is_some();
    let result = match read_value(node) {
        Some(value) if !styled => hast::Node::Text(hast::Text {
            value,
            position,
            data: None,
        }),
        _ => converter.element_with(
            "div",
            ArenaVec::new_in(converter.alloc),
            children,
            position,
        ),
    };
    apply_data(converter, data, result)
}

/// Rename, add properties to and replace the children of a converted node as its mdast
/// node's `data.hName`, `data.hProperties` and `data.hChildren` say, as `applyData` does.
pub(crate) fn apply_data<'a>(
    converter: &Converter<'a, '_>,
    data: Option<&'a Data<'a>>,
    mut result: hast::Node<'a>,
) -> hast::Node<'a> {
    if data.is_none() {
        return result;
    }
    let alloc = converter.alloc;
    if let Some(tag_name) = read_field(data, "hName").and_then(read_string) {
        result = match result {
            hast::Node::Element(mut element) => {
                element.tag_name = tag_name;
                hast::Node::Element(element)
            }
            hast::Node::Root(root) => converter.element_with(
                tag_name,
                ArenaVec::new_in(alloc),
                root.children,
                None,
            ),
            hast::Node::Unknown(hast::Unknown {
                children: Some(children),
                ..
            }) => {
                converter.element_with(tag_name, ArenaVec::new_in(alloc), children, None)
            }
            node => {
                let mut children = ArenaVec::with_capacity_in(1, alloc);
                children.push(node);
                converter.element_with(tag_name, ArenaVec::new_in(alloc), children, None)
            }
        };
    }
    if let hast::Node::Element(element) = &mut result
        && let Some(Data::Object(entries)) = read_field(data, "hProperties")
    {
        push_properties(converter, &mut element.properties, entries);
    }
    if let Some(h_children) = read_field(data, "hChildren")
        && !matches!(h_children, Data::Null)
    {
        let children = match &mut result {
            hast::Node::Element(element) => Some(&mut element.children),
            hast::Node::Root(root) => Some(&mut root.children),
            hast::Node::Unknown(unknown) => unknown.children.as_mut(),
            _ => None,
        };
        if let Some(children) = children {
            *children = match h_children {
                Data::Array(items) => convert_children(converter, items),
                _ => ArenaVec::new_in(alloc),
            };
        }
    }
    result
}

// As `structuredClone` of the node, children aside, which are converted
fn pass_through<'a>(
    converter: &Converter<'a, '_>,
    node: &mdast::Node<'a>,
    kind: &'a str,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let alloc = converter.alloc;
    let mut fields = ArenaVec::new_in(alloc);
    let text = |value: Option<&'a str>| value.map_or(Data::Null, Data::String);
    let fields: &'a [(&'a str, Data<'a>)] = match node {
        mdast::Node::Unknown(node) => node.fields,
        mdast::Node::Literal(node) => {
            fields.push(("value", Data::String(node.value)));
            &alloc.alloc(fields)[..]
        }
        mdast::Node::Math(node) => {
            fields.push(("meta", text(node.meta)));
            fields.push(("value", Data::String(node.value)));
            &alloc.alloc(fields)[..]
        }
        mdast::Node::InlineMath(node) => {
            fields.push(("value", Data::String(node.value)));
            &alloc.alloc(fields)[..]
        }
        mdast::Node::ContainerDirective(mdast::ContainerDirective {
            name,
            attributes,
            ..
        })
        | mdast::Node::LeafDirective(mdast::LeafDirective {
            name, attributes, ..
        })
        | mdast::Node::TextDirective(mdast::TextDirective {
            name, attributes, ..
        }) => {
            let mut entries = ArenaVec::with_capacity_in(attributes.len(), alloc);
            entries.extend(
                attributes
                    .iter()
                    .map(|(key, value)| (*key, Data::String(value))),
            );
            fields.push(("name", Data::String(name)));
            fields.push(("attributes", Data::Object(entries)));
            &alloc.alloc(fields)[..]
        }
        _ => &[],
    };
    hast::Node::Unknown(hast::Unknown {
        kind,
        fields,
        children: node.children().is_some().then_some(children),
        position: node.position().copied(),
        data: node.data(),
    })
}

// What `value` the unknown handler reads: the field of that name, when there is one
fn read_value<'a>(node: &mdast::Node<'a>) -> Option<&'a str> {
    match node {
        mdast::Node::Literal(node) => Some(node.value),
        mdast::Node::Math(node) => Some(node.value),
        mdast::Node::InlineMath(node) => Some(node.value),
        mdast::Node::Unknown(node) => node
            .fields
            .iter()
            .find(|(name, _)| *name == "value")
            .map(|(_, value)| read_string(value).unwrap_or_default()),
        _ => None,
    }
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

// A property already there keeps its place and takes the new value, as `Object.assign` does
fn push_properties<'a>(
    converter: &Converter<'a, '_>,
    properties: &mut ArenaVec<'a, Property<'a>>,
    entries: &'a [(&'a str, Data<'a>)],
) {
    for (name, value) in entries {
        let Some(value) = convert_property(converter, value) else {
            continue;
        };
        match properties
            .iter_mut()
            .find(|property| property.name == *name)
        {
            Some(property) => property.value = value,
            None => properties.push(Property { name, value }),
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
