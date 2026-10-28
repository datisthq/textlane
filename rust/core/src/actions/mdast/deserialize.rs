use serde_json::Value;

use crate::actions::position::deserialize::deserialize_position;
use crate::models::error::Error;
use crate::models::mdast::{Heading, Node, Paragraph, Root, Text, ThematicBreak};
use crate::{Allocator, ArenaVec};

/// Read an mdast node from unified's JSON into the arena.
pub fn deserialize_mdast<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let position = deserialize_position(value, alloc)?;
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Ok(match kind {
        "root" => Node::Root(Root {
            children: deserialize_children(value, alloc)?,
            position,
        }),
        "paragraph" => Node::Paragraph(Paragraph {
            children: deserialize_children(value, alloc)?,
            position,
        }),
        "heading" => Node::Heading(Heading {
            depth: value
                .get("depth")
                .and_then(Value::as_u64)
                .map_or(1, |depth| depth.clamp(1, 6) as u8),
            children: deserialize_children(value, alloc)?,
            position,
        }),
        "thematicBreak" => Node::ThematicBreak(ThematicBreak { position }),
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
) -> Result<ArenaVec<'a, Node<'a>>, Error> {
    let mut children = ArenaVec::new_in(alloc);
    for child in value
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        children.push(deserialize_mdast(child, alloc)?);
    }
    Ok(children)
}
