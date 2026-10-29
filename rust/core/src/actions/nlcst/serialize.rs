use serde_json::{Map, Value};

use crate::actions::hast::serialize::{insert_fields, serialize_data};
use crate::actions::mdast::serialize::MAX_JSON_DEPTH;
use crate::actions::position::serialize::serialize_position;
use crate::models::error::Error;
use crate::models::lines::Lines;
use crate::models::nlcst::Node;

/// Serialize an nlcst node to the JSON unified produces; spans need the source's `lines`.
pub fn serialize_nlcst(
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
    object.insert("type".into(), node.kind().into());
    if let Node::Unknown(unknown) = node {
        insert_fields(&mut object, unknown.fields);
    }
    if let Some(value) = node.value() {
        object.insert("value".into(), value.into());
    }
    if let Some(children) = node.children() {
        let children = children
            .iter()
            .map(|child| serialize_node(child, lines, depth + 1))
            .collect::<Result<_, _>>()?;
        object.insert("children".into(), Value::Array(children));
    }
    if let Some(data) = node.data() {
        object.insert("data".into(), serialize_data(data));
    }
    serialize_position(&mut object, node.position(), lines);
    Ok(Value::Object(object))
}
