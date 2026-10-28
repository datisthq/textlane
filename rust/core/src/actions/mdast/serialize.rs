use serde_json::{Map, Value};

use crate::actions::position::serialize::serialize_position;
use crate::models::error::Error;
use crate::models::lines::Lines;
use crate::models::mdast::Node;

// JSON values nest and drop recursively, so trees deeper than this fail instead of
// overflowing the stack; serde_json itself stops parsing at 128 levels
pub(crate) const MAX_JSON_DEPTH: usize = 1000;

/// Serialize an mdast node to the JSON unified produces; spans need the source's `lines`.
pub fn serialize_mdast(
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
    let kind = match node {
        Node::Root(_) => "root",
        Node::Paragraph(_) => "paragraph",
        Node::Heading(heading) => {
            object.insert("depth".into(), heading.depth.into());
            "heading"
        }
        Node::ThematicBreak(_) => "thematicBreak",
        Node::Emphasis(_) => "emphasis",
        Node::Strong(_) => "strong",
        Node::Break(_) => "break",
        Node::Code(code) => {
            object.insert("lang".into(), code.lang.into());
            object.insert("meta".into(), code.meta.into());
            object.insert("value".into(), code.value.into());
            "code"
        }
        Node::InlineCode(code) => {
            object.insert("value".into(), code.value.into());
            "inlineCode"
        }
        Node::Text(text) => {
            object.insert("value".into(), text.value.into());
            "text"
        }
    };
    object.insert("type".into(), kind.into());
    if let Some(children) = node.children() {
        let children = children
            .iter()
            .map(|child| serialize_node(child, lines, depth + 1))
            .collect::<Result<_, _>>()?;
        object.insert("children".into(), Value::Array(children));
    }
    serialize_position(&mut object, node.position(), lines);
    Ok(Value::Object(object))
}
