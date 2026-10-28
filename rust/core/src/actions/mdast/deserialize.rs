use serde_json::Value;

use crate::actions::mdast::serialize::MAX_JSON_DEPTH;
use crate::actions::position::deserialize::deserialize_position;
use crate::models::error::Error;
use crate::models::mdast::{
    Blockquote, Break, Code, Definition, Emphasis, Heading, Html, Image, ImageReference,
    InlineCode, Link, LinkReference, List, ListItem, Node, Paragraph, ReferenceType,
    Root, Strong, Text, ThematicBreak,
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
        "blockquote" => Node::Blockquote(Blockquote {
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "list" => Node::List(List {
            ordered: value
                .get("ordered")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            start: value
                .get("start")
                .and_then(Value::as_u64)
                .and_then(|start| u32::try_from(start).ok()),
            spread: value.get("spread").and_then(Value::as_bool),
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "listItem" => Node::ListItem(ListItem {
            spread: value.get("spread").and_then(Value::as_bool),
            checked: value.get("checked").and_then(Value::as_bool),
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "link" => Node::Link(Link {
            url: read_optional_str(value, "url", alloc).unwrap_or_default(),
            title: read_optional_str(value, "title", alloc),
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "image" => Node::Image(Image {
            url: read_optional_str(value, "url", alloc).unwrap_or_default(),
            title: read_optional_str(value, "title", alloc),
            alt: read_optional_str(value, "alt", alloc),
            position,
        }),
        "linkReference" => Node::LinkReference(LinkReference {
            identifier: read_optional_str(value, "identifier", alloc).unwrap_or_default(),
            label: read_optional_str(value, "label", alloc),
            reference_type: read_reference_type(value),
            children: deserialize_children(value, alloc, depth)?,
            position,
        }),
        "imageReference" => Node::ImageReference(ImageReference {
            identifier: read_optional_str(value, "identifier", alloc).unwrap_or_default(),
            label: read_optional_str(value, "label", alloc),
            reference_type: read_reference_type(value),
            alt: read_optional_str(value, "alt", alloc),
            position,
        }),
        "definition" => Node::Definition(Definition {
            identifier: read_optional_str(value, "identifier", alloc).unwrap_or_default(),
            label: read_optional_str(value, "label", alloc),
            url: read_optional_str(value, "url", alloc).unwrap_or_default(),
            title: read_optional_str(value, "title", alloc),
            position,
        }),
        "html" => Node::Html(Html {
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

fn read_reference_type(value: &Value) -> ReferenceType {
    match value.get("referenceType").and_then(Value::as_str) {
        Some("collapsed") => ReferenceType::Collapsed,
        Some("full") => ReferenceType::Full,
        _ => ReferenceType::Shortcut,
    }
}
