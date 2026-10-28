use serde_json::{Map, Value};

use crate::actions::position::serialize::serialize_position;
use crate::models::hast::{Node, PropertyItem, PropertyValue};
use crate::models::lines::Lines;

/// Serialize a hast node to the JSON unified produces; spans need the source's `lines`.
pub fn serialize_hast(node: &Node<'_>, lines: Option<&Lines<'_>>) -> Value {
    let mut object = Map::new();
    let serialize_all = |children: &[Node<'_>]| {
        Value::Array(
            children
                .iter()
                .map(|child| serialize_hast(child, lines))
                .collect(),
        )
    };
    match node {
        Node::Root(root) => {
            object.insert("type".into(), "root".into());
            object.insert("children".into(), serialize_all(&root.children));
        }
        Node::Element(element) => {
            object.insert("type".into(), "element".into());
            object.insert("tagName".into(), element.tag_name.into());
            let properties = element
                .properties
                .iter()
                .map(|property| {
                    (property.name.to_string(), serialize_value(&property.value))
                })
                .collect();
            object.insert("properties".into(), Value::Object(properties));
            object.insert("children".into(), serialize_all(&element.children));
        }
        Node::Text(text) => {
            object.insert("type".into(), "text".into());
            object.insert("value".into(), text.value.into());
        }
        Node::Comment(comment) => {
            object.insert("type".into(), "comment".into());
            object.insert("value".into(), comment.value.into());
        }
        Node::Doctype(_) => {
            object.insert("type".into(), "doctype".into());
        }
    }
    serialize_position(&mut object, node.position(), lines);
    Value::Object(object)
}

fn serialize_value(value: &PropertyValue<'_>) -> Value {
    match value {
        PropertyValue::Null => Value::Null,
        PropertyValue::Boolean(value) => (*value).into(),
        PropertyValue::Number(value) => (*value).into(),
        PropertyValue::String(value) => (*value).into(),
        PropertyValue::List(items) => Value::Array(
            items
                .iter()
                .map(|item| match item {
                    PropertyItem::Number(value) => (*value).into(),
                    PropertyItem::String(value) => (*value).into(),
                })
                .collect(),
        ),
    }
}
