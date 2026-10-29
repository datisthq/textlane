use serde_json::Value;

use crate::actions::hast::deserialize::{deserialize_data, deserialize_fields};
use crate::actions::mdast::serialize::MAX_JSON_DEPTH;
use crate::actions::position::deserialize::deserialize_position;
use crate::models::error::Error;
use crate::models::nlcst::{Literal, Node, Parent, Unknown};
use crate::{Allocator, ArenaVec};

/// Read an nlcst node from unified's JSON into the arena.
pub fn deserialize_nlcst<'a>(
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
    let data = match value.get("data") {
        Some(data) => Some(&*alloc.alloc(deserialize_data(data, alloc, depth + 1)?)),
        None => None,
    };
    let parent = || -> Result<Parent<'a>, Error> {
        Ok(Parent {
            children: deserialize_children(value, alloc, depth)?,
            position,
            data,
        })
    };
    let literal = || Literal {
        value: alloc.alloc_str(value.get("value").and_then(Value::as_str).unwrap_or("")),
        position,
        data,
    };
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Ok(match kind {
        "RootNode" => Node::Root(parent()?),
        "ParagraphNode" => Node::Paragraph(parent()?),
        "SentenceNode" => Node::Sentence(parent()?),
        "WordNode" => Node::Word(parent()?),
        "TextNode" => Node::Text(literal()),
        "SymbolNode" => Node::Symbol(literal()),
        "PunctuationNode" => Node::Punctuation(literal()),
        "WhiteSpaceNode" => Node::WhiteSpace(literal()),
        "SourceNode" => Node::Source(literal()),
        "" => {
            return Err(Error::new("unsupported-node", "no nlcst node `` yet"));
        }
        _ => Node::Unknown(Unknown {
            kind: alloc.alloc_str(kind),
            fields: deserialize_fields(value, alloc, depth)?,
            children: match value.get("children") {
                Some(_) => Some(deserialize_children(value, alloc, depth)?),
                None => None,
            },
            position,
            data,
        }),
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
