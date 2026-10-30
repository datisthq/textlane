use std::collections::{HashMap, HashSet};

use textlane_core::Error;
use yaml_rust2::parser::{Event, Parser};

use crate::utils::yaml::document::{Document, Item, MAX_DEPTH, Node, Scalar, yaml_error};
use crate::utils::yaml::scalar::{check_collection_tag, resolve_scalar};

#[derive(Hash, PartialEq, Eq)]
enum KeyIdentity {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
}

struct Open {
    node: usize,
    key: Option<usize>,
    keys: HashSet<KeyIdentity>,
}

/// Read YAML into a document, failing on syntax errors, more than one document, duplicate
/// keys (compared as `yaml` compares resolved scalars), circular aliases and deep nesting.
pub(crate) fn compose_document(source: &str) -> Result<Document, Error> {
    let mut parser = Parser::new_from_str(source);
    let mut document = Document::default();
    let mut anchors = HashMap::<usize, usize>::new();
    let mut stack = Vec::<Open>::new();
    let mut documents = 0;
    loop {
        let (event, _) = parser
            .next_token()
            .map_err(|error| yaml_error(error.to_string()))?;
        let (item, anchor, height) = match event {
            Event::StreamEnd => break,
            Event::DocumentStart => {
                documents += 1;
                if documents > 1 {
                    return Err(yaml_error("Source contains multiple documents"));
                }
                continue;
            }
            Event::SequenceEnd | Event::MappingEnd => {
                close_collection(&mut document, &mut stack);
                continue;
            }
            Event::Scalar(text, style, anchor, tag) => (
                Item::Scalar(resolve_scalar(text, style, tag.as_ref())?),
                anchor,
                0,
            ),
            Event::Alias(anchor) => {
                let target = anchors
                    .get(&anchor)
                    .copied()
                    .ok_or_else(|| yaml_error("Unresolved alias"))?;
                if stack.iter().any(|open| open.node == target) {
                    return Err(yaml_error("Circular alias"));
                }
                let height = document.nodes.get(target).map_or(0, |node| node.height);
                (Item::Alias(target), 0, height)
            }
            Event::SequenceStart(anchor, tag) => {
                check_collection_tag(tag.as_ref(), false)?;
                (Item::Sequence(Vec::new()), anchor, 1)
            }
            Event::MappingStart(anchor, tag) => {
                check_collection_tag(tag.as_ref(), true)?;
                (Item::Mapping(Vec::new()), anchor, 1)
            }
            _ => continue,
        };
        if stack.len() + height > MAX_DEPTH {
            return Err(yaml_error(format!(
                "Frontmatter nests deeper than {MAX_DEPTH} levels"
            )));
        }
        let id = document.nodes.len();
        let opens = matches!(item, Item::Sequence(_) | Item::Mapping(_));
        let key = match &item {
            Item::Scalar(scalar) => Some(identify_key(scalar)),
            _ => None,
        };
        document.nodes.push(Node {
            item,
            anchored: anchor != 0,
            height,
        });
        if anchor != 0 {
            anchors.insert(anchor, id);
        }
        add_child(&mut document, &mut stack, id, key)?;
        if opens {
            stack.push(Open {
                node: id,
                key: None,
                keys: HashSet::new(),
            });
        }
    }
    Ok(document)
}

fn add_child(
    document: &mut Document,
    stack: &mut [Open],
    id: usize,
    identity: Option<Option<KeyIdentity>>,
) -> Result<(), Error> {
    let Some(open) = stack.last_mut() else {
        document.root = Some(id);
        return Ok(());
    };
    let Some(parent) = document.nodes.get_mut(open.node) else {
        return Ok(());
    };
    match &mut parent.item {
        Item::Sequence(items) => items.push(id),
        Item::Mapping(pairs) => match open.key.take() {
            Some(key) => pairs.push((key, id)),
            None => {
                if let Some(Some(identity)) = identity
                    && !open.keys.insert(identity)
                {
                    return Err(yaml_error("Map keys must be unique"));
                }
                open.key = Some(id);
            }
        },
        _ => {}
    }
    Ok(())
}

fn close_collection(document: &mut Document, stack: &mut Vec<Open>) {
    let Some(open) = stack.pop() else {
        return;
    };
    let heights = |ids: &mut dyn Iterator<Item = usize>| {
        ids.filter_map(|id| document.nodes.get(id).map(|node| node.height))
            .max()
            .unwrap_or(0)
    };
    let height = match document.nodes.get(open.node).map(|node| &node.item) {
        Some(Item::Sequence(items)) => 1 + heights(&mut items.iter().copied()),
        Some(Item::Mapping(pairs)) => {
            1 + heights(&mut pairs.iter().flat_map(|&(key, value)| [key, value]))
        }
        _ => 1,
    };
    if let Some(node) = document.nodes.get_mut(open.node) {
        node.height = height;
    }
}

// `yaml` compares scalar keys by their resolved values with `===`: NaN never repeats, and
// `-0` repeats `0`
fn identify_key(scalar: &Scalar) -> Option<KeyIdentity> {
    Some(match scalar {
        Scalar::Null => KeyIdentity::Null,
        Scalar::Bool(value) => KeyIdentity::Bool(*value),
        Scalar::Number(value) if value.is_nan() => return None,
        Scalar::Number(value) => KeyIdentity::Number((value + 0.0).to_bits()),
        Scalar::String(value) => KeyIdentity::String(value.clone()),
    })
}
