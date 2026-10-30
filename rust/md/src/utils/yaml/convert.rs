use std::collections::HashMap;

use serde_json::{Map, Value};
use textlane_core::Error;

use crate::utils::yaml::document::{Document, Item, Scalar, yaml_error};
use crate::utils::yaml::number::{format_number, number_value};

/// `yaml`'s default `maxAliasCount`.
const MAX_ALIAS_COUNT: u64 = 100;
// yaml's count gives nothing but empty collections 0, so aliases of aliases of `[]` grow
// exponentially under it: the JSON nodes all aliases copy are capped as well
const MAX_EXPANDED_NODES: u64 = 1_000_000;

struct Anchor {
    count: u64,
    alias_count: Option<u64>,
    value: Option<Value>,
    size: u64,
}

enum Frame {
    Sequence {
        node: usize,
        next: usize,
        out: Vec<Value>,
    },
    Mapping {
        node: usize,
        next: usize,
        key: Option<String>,
        out: Map<String, Value>,
    },
}

struct State<'d> {
    document: &'d Document,
    anchors: HashMap<usize, Anchor>,
    expanded: u64,
}

/// The document as `yaml`'s `toJS` builds it, then as JSON: aliases expanded and counted
/// as `yaml` counts them, keys stringified as JavaScript does, array-index keys first.
pub(crate) fn convert_document(document: &Document) -> Result<Value, Error> {
    let Some(root) = document.root else {
        return Ok(Value::Null);
    };
    let mut state = State {
        document,
        anchors: HashMap::new(),
        expanded: 0,
    };
    let mut stack = Vec::<Frame>::new();
    let mut ready = state.enter(root, &mut stack)?;
    loop {
        if let Some(value) = ready.take() {
            match stack.last_mut() {
                None => return Ok(value),
                Some(Frame::Sequence { out, .. }) => out.push(value),
                Some(Frame::Mapping { out, key, .. }) => {
                    if let Some(key) = key.take() {
                        out.insert(key, value);
                    }
                }
            }
        }
        let Some(frame) = stack.last_mut() else {
            return Ok(Value::Null);
        };
        let child = match frame {
            Frame::Sequence { node, next, .. } => {
                let child = state.items(*node).get(*next).copied();
                *next += 1;
                child
            }
            Frame::Mapping {
                node, next, key, ..
            } => {
                let pair = state.pairs(*node).get(*next).copied();
                *next += 1;
                match pair {
                    Some((name, child)) => {
                        *key = Some(state.key_text(name)?);
                        Some(child)
                    }
                    None => None,
                }
            }
        };
        if let Some(child) = child {
            ready = state.enter(child, &mut stack)?;
            continue;
        }
        let (node, value) = match stack.pop() {
            Some(Frame::Sequence { node, out, .. }) => (node, Value::Array(out)),
            Some(Frame::Mapping { node, out, .. }) => (node, order_keys(out)),
            None => return Ok(Value::Null),
        };
        if let Some(anchor) = state.anchors.get_mut(&node) {
            anchor.size = count_nodes(&value);
            anchor.value = Some(value.clone());
        }
        ready = Some(value);
    }
}

impl State<'_> {
    fn items(&self, node: usize) -> &[usize] {
        match self.document.nodes.get(node).map(|node| &node.item) {
            Some(Item::Sequence(items)) => items,
            _ => &[],
        }
    }

    fn pairs(&self, node: usize) -> &[(usize, usize)] {
        match self.document.nodes.get(node).map(|node| &node.item) {
            Some(Item::Mapping(pairs)) => pairs,
            _ => &[],
        }
    }

    // A scalar or alias is its value; a collection opens a frame and is finished later
    fn enter(
        &mut self,
        id: usize,
        stack: &mut Vec<Frame>,
    ) -> Result<Option<Value>, Error> {
        let Some(node) = self.document.nodes.get(id) else {
            return Ok(Some(Value::Null));
        };
        if node.anchored {
            self.anchors.insert(
                id,
                Anchor {
                    count: 1,
                    alias_count: None,
                    value: None,
                    size: 0,
                },
            );
        }
        let value = match &node.item {
            Item::Scalar(scalar) => scalar_value(scalar),
            Item::Alias(target) => self.resolve_alias(*target)?,
            Item::Sequence(_) => {
                stack.push(Frame::Sequence {
                    node: id,
                    next: 0,
                    out: Vec::new(),
                });
                return Ok(None);
            }
            Item::Mapping(_) => {
                stack.push(Frame::Mapping {
                    node: id,
                    next: 0,
                    key: None,
                    out: Map::new(),
                });
                return Ok(None);
            }
        };
        if let Some(anchor) = self.anchors.get_mut(&id) {
            anchor.size = count_nodes(&value);
            anchor.value = Some(value.clone());
        }
        Ok(Some(value))
    }

    fn key_text(&mut self, id: usize) -> Result<String, Error> {
        let scalar = match self.document.nodes.get(id).map(|node| &node.item) {
            Some(Item::Scalar(scalar)) => {
                self.enter(id, &mut Vec::new())?;
                scalar
            }
            Some(Item::Alias(target)) => {
                self.resolve_alias(*target)?;
                match self.document.nodes.get(*target).map(|node| &node.item) {
                    Some(Item::Scalar(scalar)) => scalar,
                    _ => return Err(collection_key_error()),
                }
            }
            _ => return Err(collection_key_error()),
        };
        Ok(match scalar {
            Scalar::Null => String::new(),
            Scalar::Bool(value) => value.to_string(),
            Scalar::Number(value) => format_number(*value),
            Scalar::String(value) => value.clone(),
        })
    }

    fn resolve_alias(&mut self, target: usize) -> Result<Value, Error> {
        let alias_count = match self.anchors.get(&target) {
            None => return Err(yaml_error("Unresolved alias")),
            Some(Anchor { value: None, .. }) => return Err(yaml_error("Circular alias")),
            Some(anchor) => anchor
                .alias_count
                .unwrap_or_else(|| self.count_aliases(target)),
        };
        let Some(anchor) = self.anchors.get_mut(&target) else {
            return Err(yaml_error("Unresolved alias"));
        };
        anchor.count += 1;
        anchor.alias_count = Some(alias_count);
        self.expanded = self.expanded.saturating_add(anchor.size);
        if anchor.count.saturating_mul(alias_count) > MAX_ALIAS_COUNT
            || self.expanded > MAX_EXPANDED_NODES
        {
            return Err(yaml_error(
                "Excessive alias count indicates a resource exhaustion attack",
            ));
        }
        Ok(anchor.value.clone().unwrap_or(Value::Null))
    }

    // `yaml`'s `getAliasCount`: the largest of a scalar's 1 and each inner alias's count
    // times its own alias count, 0 for nothing but empty collections
    fn count_aliases(&self, node: usize) -> u64 {
        let mut largest = 0;
        let mut stack = vec![node];
        while let Some(id) = stack.pop() {
            let count = match self.document.nodes.get(id).map(|node| &node.item) {
                Some(Item::Scalar(_)) => 1,
                Some(Item::Alias(target)) => {
                    self.anchors.get(target).map_or(0, |anchor| {
                        anchor.count.saturating_mul(anchor.alias_count.unwrap_or(0))
                    })
                }
                Some(Item::Sequence(items)) => {
                    stack.extend(items);
                    0
                }
                Some(Item::Mapping(pairs)) => {
                    stack.extend(pairs.iter().flat_map(|&(key, value)| [key, value]));
                    0
                }
                None => 0,
            };
            largest = largest.max(count);
        }
        largest
    }
}

// How many JSON values a value holds, itself included, without recursing
fn count_nodes(value: &Value) -> u64 {
    let mut count = 0;
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        count += 1;
        match value {
            Value::Array(items) => stack.extend(items),
            Value::Object(fields) => stack.extend(fields.values()),
            _ => {}
        }
    }
    count
}

fn scalar_value(scalar: &Scalar) -> Value {
    match scalar {
        Scalar::Null => Value::Null,
        Scalar::Bool(value) => Value::Bool(*value),
        Scalar::Number(value) => number_value(*value),
        Scalar::String(value) => Value::String(value.clone()),
    }
}

fn collection_key_error() -> Error {
    yaml_error("Keys with collection values are not supported")
}

// JavaScript enumerates an object's array-index keys first, in numeric order
fn order_keys(map: Map<String, Value>) -> Value {
    if !map.keys().any(|key| array_index(key).is_some()) {
        return Value::Object(map);
    }
    let (mut indexes, rest): (Vec<_>, Vec<_>) = map
        .into_iter()
        .partition(|(key, _)| array_index(key).is_some());
    indexes.sort_by_key(|(key, _)| array_index(key));
    Value::Object(indexes.into_iter().chain(rest).collect())
}

fn array_index(key: &str) -> Option<u32> {
    if key.len() > 1 && key.starts_with('0') || !key.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    key.parse::<u32>().ok().filter(|index| *index != u32::MAX)
}
