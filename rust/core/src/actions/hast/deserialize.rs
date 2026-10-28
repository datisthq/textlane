use serde_json::Value;

use crate::actions::position::deserialize::deserialize_position;
use crate::models::error::Error;
use crate::models::hast::{
    Comment, Doctype, Element, Node, Property, PropertyItem, PropertyValue, Root, Text,
};
use crate::{Allocator, ArenaVec};

/// Read a hast node from unified's JSON into the arena.
pub fn deserialize_hast<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let position = deserialize_position(value, alloc)?;
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let read_str =
        |key: &str| alloc.alloc_str(value.get(key).and_then(Value::as_str).unwrap_or(""));
    Ok(match kind {
        "root" => Node::Root(Root {
            children: deserialize_children(value, alloc)?,
            position,
        }),
        "element" => Node::Element(Element {
            tag_name: read_str("tagName"),
            properties: deserialize_properties(value, alloc)?,
            children: deserialize_children(value, alloc)?,
            position,
        }),
        "text" => Node::Text(Text {
            value: read_str("value"),
            position,
        }),
        "comment" => Node::Comment(Comment {
            value: read_str("value"),
            position,
        }),
        "doctype" => Node::Doctype(Doctype { position }),
        _ => {
            return Err(Error::new(
                "unsupported-node",
                format!("no hast node `{kind}` yet"),
            ));
        }
    })
}

fn deserialize_children<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Result<ArenaVec<'a, Node<'a>>, Error> {
    let mut children = ArenaVec::new_in(alloc);
    for child in value
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        children.push(deserialize_hast(child, alloc)?);
    }
    Ok(children)
}

fn deserialize_properties<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Result<ArenaVec<'a, Property<'a>>, Error> {
    let mut properties = ArenaVec::new_in(alloc);
    for (name, value) in value
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let value = match value {
            Value::Null => PropertyValue::Null,
            Value::Bool(value) => PropertyValue::Boolean(*value),
            Value::Number(value) => {
                PropertyValue::Number(value.as_f64().unwrap_or_default())
            }
            Value::String(value) => PropertyValue::String(alloc.alloc_str(value)),
            Value::Array(items) => {
                let mut list = ArenaVec::new_in(alloc);
                for item in items {
                    list.push(match item {
                        Value::Number(value) => {
                            PropertyItem::Number(value.as_f64().unwrap_or_default())
                        }
                        Value::String(value) => {
                            PropertyItem::String(alloc.alloc_str(value))
                        }
                        _ => {
                            return Err(Error::new(
                                "invalid-tree",
                                format!("not a property list item: {item}"),
                            ));
                        }
                    });
                }
                PropertyValue::List(list)
            }
            Value::Object(_) => {
                return Err(Error::new(
                    "invalid-tree",
                    format!("not a property value: {value}"),
                ));
            }
        };
        properties.push(Property {
            name: alloc.alloc_str(name),
            value,
        });
    }
    Ok(properties)
}
