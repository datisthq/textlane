use serde_json::Value;

use crate::actions::mdast::serialize::MAX_JSON_DEPTH;
use crate::actions::position::deserialize::deserialize_position;
use crate::models::error::Error;
use crate::models::mdast::{
    Break, Code, Emphasis, Heading, InlineCode, Node, Paragraph, Root, Strong, Text,
    ThematicBreak,
};
use crate::{Allocator, ArenaVec};

/// Read an mdast node from unified's JSON into the arena.
pub fn deserialize_mdast<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    deserialize_node(value, alloc, 0)
}

fn deserialize_node<'a>(
    value: &Value,
    alloc: &'a Allocator,
    depth: usize,
) -> Result<Node<'a>, Error> {
    if depth > MAX_JSON_DEPTH {
        return Err(Error::new(
            "tree-too-deep",
            format!("trees nest at most {MAX_JSON_DEPTH} levels in JSON"),
        ));
    }
    let position = deserialize_position(value, alloc)?;
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Ok(match kind {
        "root" => Node::Root(Root {
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "paragraph" => Node::Paragraph(Paragraph {
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "heading" => Node::Heading(Heading {
            depth: value
                .get("depth")
                .and_then(Value::as_u64)
                .map_or(1, |depth| depth.clamp(1, 6) as u8),
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "thematicBreak" => Node::ThematicBreak(ThematicBreak { position }),
        "break" => Node::Break(Break { position }),
        "code" => Node::Code(Code {
            lang: read_optional_str(value, "lang", alloc),
            meta: read_optional_str(value, "meta", alloc),
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
        }),
        "inlineCode" => Node::InlineCode(InlineCode {
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
        }),
        "emphasis" => Node::Emphasis(Emphasis {
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "strong" => Node::Strong(Strong {
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "text" => Node::Text(Text {
            value: alloc
                .alloc_str(value.get("value").and_then(Value::as_str).unwrap_or("")),
            position,
        }),
        _ => {
            return Err(Error::new(
                "unsupported-node",
                format!("no mdast node `{kind}` yet"),
            ));
        }
    })
}

fn deserialize_children<'a>(
    value: &Value,
    alloc: &'a Allocator,
    depth: usize,
) -> Result<ArenaVec<'a, Node<'a>>, Error> {
    let mut children = ArenaVec::new_in(alloc);
    for child in value
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        children.push(deserialize_node(child, alloc, depth + 1)?);
    }
    Ok(children)
}

fn read_optional_str<'a>(
    value: &Value,
    key: &str,
    alloc: &'a Allocator,
) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(|text| &*alloc.alloc_str(text))
}
