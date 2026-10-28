use serde_json::{Map, Value};

use crate::actions::mdast::serialize::MAX_JSON_DEPTH;
use crate::actions::position::serialize::serialize_position;
use crate::models::data::Data;
use crate::models::error::Error;
use crate::models::hast::{Node, PropertyItem, PropertyValue, Root};
use crate::models::lines::Lines;

/// Serialize a hast node to the JSON unified produces; spans need the source's `lines`.
pub fn serialize_hast(
    node: &Node<'_>,
    lines: Option<&Lines<'_>>,
) -> Result<Value, Error> {
    serialize_node(node, lines, 0)
}

fn serialize_node(
    node: &Node<'_>,
    lines: Option<&Lines<'_>>,
    depth: usize,
) -> Result<Value, Error> {
    if depth > MAX_JSON_DEPTH {
        return Err(Error::new(
            "tree-too-deep",
            format!("trees nest at most {MAX_JSON_DEPTH} levels in JSON"),
        ));
    }
    let mut object = Map::new();
    let serialize_all = |children: &[Node<'_>]| -> Result<Value, Error> {
        let children = children
            .iter()
            .map(|child| serialize_node(child, lines, depth + 1))
            .collect::<Result<_, _>>()?;
        Ok(Value::Array(children))
    };
    match node {
        Node::Root(root) => return serialize_root(root, lines, depth),
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
            object.insert("children".into(), serialize_all(&element.children)?);
            if let Some(content) = element.content {
                object
                    .insert("content".into(), serialize_root(content, lines, depth + 1)?);
            }
        }
        Node::Text(text) => {
            object.insert("type".into(), "text".into());
            object.insert("value".into(), text.value.into());
        }
        Node::Comment(comment) => {
            object.insert("type".into(), "comment".into());
            object.insert("value".into(), comment.value.into());
        }
        Node::Raw(raw) => {
            object.insert("type".into(), "raw".into());
            object.insert("value".into(), raw.value.into());
        }
        Node::Doctype(_) => {
            object.insert("type".into(), "doctype".into());
        }
    }
    if let Some(data) = node.data() {
        object.insert("data".into(), serialize_data(data));
    }
    serialize_position(&mut object, node.position(), lines);
    Ok(Value::Object(object))
}

fn serialize_root(
    root: &Root<'_>,
    lines: Option<&Lines<'_>>,
    depth: usize,
) -> Result<Value, Error> {
    if depth > MAX_JSON_DEPTH {
        return Err(Error::new(
            "tree-too-deep",
            format!("trees nest at most {MAX_JSON_DEPTH} levels in JSON"),
        ));
    }
    let mut object = Map::new();
    object.insert("type".into(), "root".into());
    let children = root
        .children
        .iter()
        .map(|child| serialize_node(child, lines, depth + 1))
        .collect::<Result<_, _>>()?;
    object.insert("children".into(), Value::Array(children));
    let mut data = match root.data.map(serialize_data) {
        Some(Value::Object(data)) => data,
        _ => Map::new(),
    };
    if let Some(quirks_mode) = root.quirks_mode {
        data.insert("quirksMode".into(), quirks_mode.into());
    }
    if !data.is_empty() {
        object.insert("data".into(), Value::Object(data));
    }
    serialize_position(&mut object, root.position.as_ref(), lines);
    Ok(Value::Object(object))
}

/// Write unist `data` as JSON.
pub(crate) fn serialize_data(data: &Data<'_>) -> Value {
    match data {
        Data::Null => Value::Null,
        Data::Boolean(value) => (*value).into(),
        Data::Number(value) => serialize_number(*value),
        Data::String(value) => (*value).into(),
        Data::Array(items) => Value::Array(items.iter().map(serialize_data).collect()),
        Data::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.to_string(), serialize_data(value)))
                .collect(),
        ),
    }
}

fn serialize_value(value: &PropertyValue<'_>) -> Value {
    match value {
        PropertyValue::Null => Value::Null,
        PropertyValue::Boolean(value) => (*value).into(),
        PropertyValue::Number(value) => serialize_number(*value),
        PropertyValue::String(value) => (*value).into(),
        PropertyValue::List(items) => Value::Array(
            items
                .iter()
                .map(|item| match item {
                    PropertyItem::Number(value) => serialize_number(*value),
                    PropertyItem::String(value) => (*value).into(),
                })
                .collect(),
        ),
    }
}

// Whole numbers serialize as integers, as JavaScript writes them
fn serialize_number(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
        Value::from(value as i64)
    } else {
        Value::from(value)
    }
}
