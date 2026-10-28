use serde_json::{Map, Value};

use crate::actions::position::serialize::serialize_position;
use crate::models::lines::Lines;
use crate::models::mdast::Node;

/// Serialize an mdast node to the JSON unified produces; spans need the source's `lines`.
pub fn serialize_mdast(node: &Node<'_>, lines: Option<&Lines<'_>>) -> Value {
    let mut object = Map::new();
    let kind = match node {
        Node::Root(_) => "root",
        Node::Paragraph(_) => "paragraph",
        Node::Heading(heading) => {
            object.insert("depth".into(), heading.depth.into());
            "heading"
        }
        Node::ThematicBreak(_) => "thematicBreak",
        Node::Text(text) => {
            object.insert("value".into(), text.value.into());
            "text"
        }
    };
    object.insert("type".into(), kind.into());
    if let Some(children) = node.children() {
        let children = children
            .iter()
            .map(|child| serialize_mdast(child, lines))
            .collect();
        object.insert("children".into(), Value::Array(children));
    }
    serialize_position(&mut object, node.position(), lines);
    Value::Object(object)
}
