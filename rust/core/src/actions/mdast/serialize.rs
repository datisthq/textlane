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
        Node::Blockquote(_) => "blockquote",
        Node::List(list) => {
            object.insert("ordered".into(), list.ordered.into());
            object.insert("start".into(), list.start.into());
            if let Some(spread) = list.spread {
                object.insert("spread".into(), spread.into());
            }
            "list"
        }
        Node::ListItem(item) => {
            if let Some(spread) = item.spread {
                object.insert("spread".into(), spread.into());
            }
            object.insert("checked".into(), item.checked.into());
            "listItem"
        }
        Node::Link(link) => {
            object.insert("url".into(), link.url.into());
            object.insert("title".into(), link.title.into());
            "link"
        }
        Node::Image(image) => {
            object.insert("url".into(), image.url.into());
            object.insert("title".into(), image.title.into());
            object.insert("alt".into(), image.alt.into());
            "image"
        }
        Node::LinkReference(reference) => {
            object.insert("identifier".into(), reference.identifier.into());
            object.insert("label".into(), reference.label.into());
            object.insert(
                "referenceType".into(),
                reference.reference_type.as_str().into(),
            );
            "linkReference"
        }
        Node::ImageReference(reference) => {
            object.insert("identifier".into(), reference.identifier.into());
            object.insert("label".into(), reference.label.into());
            object.insert(
                "referenceType".into(),
                reference.reference_type.as_str().into(),
            );
            object.insert("alt".into(), reference.alt.into());
            "imageReference"
        }
        Node::Definition(definition) => {
            object.insert("identifier".into(), definition.identifier.into());
            object.insert("label".into(), definition.label.into());
            object.insert("url".into(), definition.url.into());
            object.insert("title".into(), definition.title.into());
            "definition"
        }
        Node::Html(html) => {
            object.insert("value".into(), html.value.into());
            "html"
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
