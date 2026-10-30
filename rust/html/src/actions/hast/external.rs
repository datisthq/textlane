use serde_json::Value;
use textlane_core::hast::{
    Element, Node, Property, PropertyItem, PropertyValue, deserialize_hast,
};
use textlane_core::{Allocator, ArenaVec, Error};

use crate::models::options::{ExternalLinks, Tokens};

/// Mark each link to another site as rehype-external-links does: its properties, `rel`,
/// `target` and appended content, in that order.
pub fn mark_external_links<'a>(
    node: &mut Node<'a>,
    options: &ExternalLinks,
    alloc: &'a Allocator,
) -> Result<(), Error> {
    let protocols: Vec<&str> = match &options.protocols {
        Some(protocols) => protocols.iter().map(String::as_str).collect(),
        None => vec!["http", "https"],
    };
    let rel = read_rel(options);
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                if element.tag_name == "a" && is_external(element, &protocols) {
                    mark_link(element, options, &rel, alloc)?;
                }
                &mut element.children
            }
            _ => continue,
        };
        stack.extend(children.iter_mut().rev());
    }
    Ok(())
}

fn mark_link<'a>(
    element: &mut Element<'a>,
    options: &ExternalLinks,
    rel: &[String],
    alloc: &'a Allocator,
) -> Result<(), Error> {
    if let Some(properties) = &options.properties {
        for (name, value) in properties {
            set_property(element, alloc.alloc_str(name), read_value(value, alloc));
        }
    }
    if !rel.is_empty() {
        let mut items = ArenaVec::with_capacity_in(rel.len(), alloc);
        items.extend(
            rel.iter()
                .map(|token| PropertyItem::String(alloc.alloc_str(token))),
        );
        set_property(element, "rel", PropertyValue::List(items));
    }
    if let Some(target) = options
        .target
        .as_deref()
        .filter(|target| !target.is_empty())
    {
        set_property(
            element,
            "target",
            PropertyValue::String(alloc.alloc_str(target)),
        );
    }
    if let Some(content) = &options.content {
        let nodes: Vec<&Value> = match content {
            Value::Array(nodes) => nodes.iter().collect(),
            Value::Null | Value::Bool(false) => return Ok(()),
            node => vec![node],
        };
        let mut children = ArenaVec::with_capacity_in(nodes.len(), alloc);
        for node in nodes {
            children.push(deserialize_hast(node, alloc)?);
        }
        let mut properties = ArenaVec::new_in(alloc);
        for (name, value) in options.content_properties.iter().flatten() {
            properties.push(Property {
                name: alloc.alloc_str(name),
                value: read_value(value, alloc),
            });
        }
        element.children.push(Node::Element(Element {
            tag_name: "span",
            properties,
            children,
            position: None,
            data: None,
            content: None,
        }));
    }
    Ok(())
}

// An absolute URL in one of the protocols, or a protocol-relative one; a Windows path is
// not absolute, as is-absolute-url decides
fn is_external(element: &Element<'_>, protocols: &[&str]) -> bool {
    let href = element
        .properties
        .iter()
        .find_map(|property| match property.value {
            PropertyValue::String(href) if property.name == "href" => Some(href),
            _ => None,
        });
    let Some(url) = href else { return false };
    match is_absolute(url) {
        true => url
            .split_once(':')
            .is_some_and(|(protocol, _)| protocols.contains(&protocol)),
        false => url.starts_with("//"),
    }
}

// is-absolute-url: a scheme, a letter then letters, digits, `+`, `-` or `.`, up to the first
// `:`, unless the URL is a Windows path such as `C:\`
fn is_absolute(url: &str) -> bool {
    let bytes = url.as_bytes();
    if bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && &bytes[1..3] == b":\\" {
        return false;
    }
    let Some(colon) = url.find(':') else {
        return false;
    };
    let scheme = &bytes[..colon];
    scheme.first().is_some_and(u8::is_ascii_alphabetic)
        && scheme
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(byte))
}

// `rel` falls back to `nofollow` when it is unset or an empty string, as the JavaScript `||`
// does, and a string splits as space-separated-tokens splits it
fn read_rel(options: &ExternalLinks) -> Vec<String> {
    let tokens = |text: &str| -> Vec<String> {
        text.split([' ', '\t', '\n', '\r', '\x0C'])
            .filter(|token| !token.is_empty())
            .map(str::to_owned)
            .collect()
    };
    match &options.rel {
        Some(Tokens::One(text)) if !text.is_empty() => tokens(text),
        Some(Tokens::Many(list)) => list.clone(),
        _ => vec!["nofollow".to_owned()],
    }
}

fn set_property<'a>(element: &mut Element<'a>, name: &'a str, value: PropertyValue<'a>) {
    match element
        .properties
        .iter_mut()
        .find(|property| property.name == name)
    {
        Some(property) => property.value = value,
        None => element.properties.push(Property { name, value }),
    }
}

fn read_value<'a>(value: &Value, alloc: &'a Allocator) -> PropertyValue<'a> {
    match value {
        Value::Bool(value) => PropertyValue::Boolean(*value),
        Value::Number(number) => {
            PropertyValue::Number(number.as_f64().unwrap_or_default())
        }
        Value::String(text) => PropertyValue::String(alloc.alloc_str(text)),
        Value::Array(items) => {
            let mut list = ArenaVec::with_capacity_in(items.len(), alloc);
            for item in items {
                match item {
                    Value::Number(number) => {
                        list.push(PropertyItem::Number(
                            number.as_f64().unwrap_or_default(),
                        ));
                    }
                    Value::String(text) => {
                        list.push(PropertyItem::String(alloc.alloc_str(text)))
                    }
                    _ => {}
                }
            }
            PropertyValue::List(list)
        }
        Value::Null | Value::Object(_) => PropertyValue::Null,
    }
}
