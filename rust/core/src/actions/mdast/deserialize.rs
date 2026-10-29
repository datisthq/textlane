use serde_json::Value;

use crate::actions::hast::deserialize::{deserialize_data, deserialize_fields};
use crate::actions::mdast::serialize::MAX_JSON_DEPTH;
use crate::actions::position::deserialize::deserialize_position;
use crate::models::error::Error;
use crate::models::mdast::{
    AlignKind, Blockquote, Break, Code, ContainerDirective, DefList, DefListDescription,
    DefListTerm, Definition, Delete, DirectiveAttributes, Emphasis, FootnoteDefinition,
    FootnoteReference, Heading, Html, Image, ImageReference, InlineCode, InlineMath,
    LeafDirective, Link, LinkReference, List, ListItem, Literal, Math, Node, Paragraph,
    ReferenceType, Root, Strong, Table, TableCell, TableRow, Text, TextDirective,
    ThematicBreak, Toml, Unknown, Yaml,
};
use crate::{Allocator, ArenaVec, Data};

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
    let data = read_data(value, alloc, depth)?;
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Ok(match kind {
        "root" => Node::Root(Root {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "paragraph" => Node::Paragraph(Paragraph {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "heading" => Node::Heading(Heading {
            depth: value
                .get("depth")
                .and_then(Value::as_u64)
                .map_or(1, |depth| depth.clamp(1, 6) as u8),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "thematicBreak" => Node::ThematicBreak(ThematicBreak { position, data }),
        "break" => Node::Break(Break { position, data }),
        "code" => Node::Code(Code {
            lang: read_optional_str(value, "lang", alloc),
            meta: read_optional_str(value, "meta", alloc),
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        "blockquote" => Node::Blockquote(Blockquote {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
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
            data,
        }),
        "listItem" => Node::ListItem(ListItem {
            spread: value.get("spread").and_then(Value::as_bool),
            checked: value.get("checked").and_then(Value::as_bool),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "link" => Node::Link(Link {
            url: read_optional_str(value, "url", alloc).unwrap_or_default(),
            title: read_optional_str(value, "title", alloc),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "image" => Node::Image(Image {
            url: read_optional_str(value, "url", alloc).unwrap_or_default(),
            title: read_optional_str(value, "title", alloc),
            alt: read_optional_str(value, "alt", alloc),
            position,
            data,
        }),
        "linkReference" => Node::LinkReference(LinkReference {
            identifier: read_optional_str(value, "identifier", alloc).unwrap_or_default(),
            label: read_optional_str(value, "label", alloc),
            reference_type: read_reference_type(value),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "imageReference" => Node::ImageReference(ImageReference {
            identifier: read_optional_str(value, "identifier", alloc).unwrap_or_default(),
            label: read_optional_str(value, "label", alloc),
            reference_type: read_reference_type(value),
            alt: read_optional_str(value, "alt", alloc),
            position,
            data,
        }),
        "definition" => Node::Definition(Definition {
            identifier: read_optional_str(value, "identifier", alloc).unwrap_or_default(),
            label: read_optional_str(value, "label", alloc),
            url: read_optional_str(value, "url", alloc).unwrap_or_default(),
            title: read_optional_str(value, "title", alloc),
            position,
            data,
        }),
        "delete" => Node::Delete(Delete {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "table" => Node::Table(Table {
            align: {
                let mut align = ArenaVec::new_in(alloc);
                for item in value
                    .get("align")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    align.push(match item.as_str() {
                        Some("left") => AlignKind::Left,
                        Some("right") => AlignKind::Right,
                        Some("center") => AlignKind::Center,
                        _ => AlignKind::None,
                    });
                }
                align
            },
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "footnoteDefinition" => Node::FootnoteDefinition(FootnoteDefinition {
            identifier: read_identifier(value, alloc),
            label: read_optional_str(value, "label", alloc),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "footnoteReference" => Node::FootnoteReference(FootnoteReference {
            identifier: read_identifier(value, alloc),
            label: read_optional_str(value, "label", alloc),
            position,
            data,
        }),
        "containerDirective" => Node::ContainerDirective(ContainerDirective {
            name: read_optional_str(value, "name", alloc).unwrap_or_default(),
            attributes: read_attributes(value, alloc),
            label: read_directive_label(value),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "leafDirective" => Node::LeafDirective(LeafDirective {
            name: read_optional_str(value, "name", alloc).unwrap_or_default(),
            attributes: read_attributes(value, alloc),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "textDirective" => Node::TextDirective(TextDirective {
            name: read_optional_str(value, "name", alloc).unwrap_or_default(),
            attributes: read_attributes(value, alloc),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "tableRow" => Node::TableRow(TableRow {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "tableCell" => Node::TableCell(TableCell {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "defList" => Node::DefList(DefList {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "defListTerm" => Node::DefListTerm(DefListTerm {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "defListDescription" => Node::DefListDescription(DefListDescription {
            spread: value
                .get("spread")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "html" => Node::Html(Html {
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        "inlineCode" => Node::InlineCode(InlineCode {
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        "math" => Node::Math(Math {
            meta: read_optional_str(value, "meta", alloc),
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        "inlineMath" => Node::InlineMath(InlineMath {
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        "emphasis" => Node::Emphasis(Emphasis {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "strong" => Node::Strong(Strong {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        }),
        "text" => Node::Text(Text {
            value: alloc
                .alloc_str(value.get("value").and_then(Value::as_str).unwrap_or("")),
            position,
            data,
        }),
        "yaml" => Node::Yaml(Yaml {
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        "toml" => Node::Toml(Toml {
            value: read_optional_str(value, "value", alloc).unwrap_or_default(),
            position,
            data,
        }),
        _ if !kind.is_empty()
            && let Some(object) = value.as_object()
            && let Some(text) = value.get("value").and_then(Value::as_str)
            && object.keys().all(|key| {
                matches!(key.as_str(), "type" | "value" | "position" | "data")
            }) =>
        {
            Node::Literal(Literal {
                kind: alloc.alloc_str(kind),
                value: alloc.alloc_str(text),
                position,
                data,
            })
        }
        _ if !kind.is_empty() => Node::Unknown(Unknown {
            kind: alloc.alloc_str(kind),
            fields: deserialize_fields(value, alloc, depth)?,
            children: match value.get("children") {
                Some(_) => Some(deserialize_children(value, alloc, depth)?),
                None => None,
            },
            position,
            data,
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

fn read_data<'a>(
    value: &Value,
    alloc: &'a Allocator,
    depth: usize,
) -> Result<Option<&'a Data<'a>>, Error> {
    match value.get("data") {
        Some(data) => Ok(Some(&*alloc.alloc(deserialize_data(
            data,
            alloc,
            depth + 1,
        )?))),
        None => Ok(None),
    }
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

// mdast-util-to-hast reads a footnote's identifier through `String()`, so a number works
fn read_identifier<'a>(value: &Value, alloc: &'a Allocator) -> &'a str {
    match value.get("identifier") {
        Some(Value::Number(number)) => alloc.alloc_str(&number.to_string()),
        _ => read_optional_str(value, "identifier", alloc).unwrap_or_default(),
    }
}

// mdast-util-to-markdown writes a reference without a type as a collapsed one
fn read_reference_type(value: &Value) -> ReferenceType {
    match value.get("referenceType").and_then(Value::as_str) {
        Some("shortcut") => ReferenceType::Shortcut,
        Some("full") => ReferenceType::Full,
        _ => ReferenceType::Collapsed,
    }
}

// mdast-util-directive writes each value through `String()` and skips null ones
fn read_attributes<'a>(value: &Value, alloc: &'a Allocator) -> DirectiveAttributes<'a> {
    let mut attributes = ArenaVec::new_in(alloc);
    for (key, item) in value
        .get("attributes")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let text = match item {
            Value::String(text) => text.clone(),
            Value::Bool(flag) => flag.to_string(),
            Value::Number(number) => match number.as_f64() {
                Some(float) if float.fract() == 0.0 && float.abs() < 1e21 => {
                    format!("{float:.0}")
                }
                _ => number.to_string(),
            },
            _ => continue,
        };
        attributes.push((&*alloc.alloc_str(key), &*alloc.alloc_str(&text)));
    }
    attributes
}

fn read_directive_label(value: &Value) -> bool {
    value
        .get("children")
        .and_then(Value::as_array)
        .and_then(|children| children.first())
        .is_some_and(|head| {
            head.get("type").and_then(Value::as_str) == Some("paragraph")
                && head.pointer("/data/directiveLabel").is_some_and(|label| {
                    !matches!(label, Value::Null | Value::Bool(false))
                })
        })
}
