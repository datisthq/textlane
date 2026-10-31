use textlane_core::Data;

use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};

/// The ESTree program a plugin stored as an MDX node's `data.estree`, read into the store.
pub(crate) struct Loaded {
    pub(crate) program: Id,
    pub(crate) comments: Vec<Id>,
    /// How far its offsets reach past `base`.
    pub(crate) span: usize,
}

enum Frame<'d> {
    Object {
        entries: &'d [(&'d str, Data<'d>)],
        next: usize,
        kind: Kind,
        fields: Vec<(Key, Value)>,
        key: Key,
    },
    Array {
        items: &'d [Data<'d>],
        values: Vec<Value>,
    },
}

impl<'d> Frame<'d> {
    // Document lines and `range` mean nothing once the offsets are moved
    fn next_child(&mut self, store: &mut Store) -> Option<(Option<Key>, &'d Data<'d>)> {
        match self {
            Frame::Object {
                entries, next, key, ..
            } => {
                while let Some((name, value)) = entries.get(*next) {
                    *next += 1;
                    if !matches!(*name, "type" | "loc" | "range") {
                        *key = store.intern_key(name);
                        return Some((Some(*key), value));
                    }
                }
                None
            }
            Frame::Array { items, values } => {
                items.get(values.len()).map(|item| (None, item))
            }
        }
    }

    fn push(&mut self, value: Value) {
        match self {
            Frame::Object { fields, key, .. } => fields.push((*key, value)),
            Frame::Array { values, .. } => values.push(value),
        }
    }

    fn finish(self, store: &mut Store) -> Value {
        match self {
            Frame::Object { kind, fields, .. } => Value::Node(store.add(kind, fields)),
            Frame::Array { values, .. } => Value::List(values),
        }
    }
}

/// Read `data.estree` as hast-util-to-estree takes it, every `start`/`end` moved by `base`;
/// nothing when the node has none, so its `value` is parsed instead.
pub(crate) fn load_estree(
    store: &mut Store,
    data: Option<&Data<'_>>,
    base: usize,
) -> Option<Loaded> {
    let Some(Data::Object(entries)) = data else {
        return None;
    };
    let estree = entries
        .iter()
        .find(|(key, _)| *key == "estree")
        .map(|(_, value)| value)
        .filter(|value| matches!(value, Data::Object(_)))?;
    let mut span = 0;
    let program = read_value(store, estree, base, &mut span).as_node()?;
    let comments = store.children(program, Key::COMMENTS);
    store.remove(program, Key::COMMENTS);
    Some(Loaded {
        program,
        comments,
        span,
    })
}

// `data` nests as deep as the script does, so values are built on an explicit stack
fn read_value(
    store: &mut Store,
    root: &Data<'_>,
    base: usize,
    span: &mut usize,
) -> Value {
    let mut stack: Vec<Frame<'_>> = Vec::new();
    let (mut key, mut data) = (None, root);
    loop {
        let mut value = match data {
            Data::Null => Some(Value::Null),
            Data::Boolean(value) => Some(Value::Bool(*value)),
            Data::Number(value) if matches!(key, Some(Key::START | Key::END)) => {
                *span = (*span).max(*value as usize);
                Some(Value::Number(value + base as f64))
            }
            Data::Number(value) => Some(Value::Number(*value)),
            Data::String(value) => Some(Value::str(value)),
            Data::Array(items) => {
                stack.push(Frame::Array {
                    items,
                    values: Vec::new(),
                });
                None
            }
            Data::Object(entries) => {
                let kind = entries
                    .iter()
                    .find_map(|(name, value)| match (*name, value) {
                        ("type", Data::String(text)) => Some(store.intern_kind(text)),
                        _ => None,
                    })
                    .unwrap_or(Kind::NONE);
                stack.push(Frame::Object {
                    entries,
                    next: 0,
                    kind,
                    fields: Vec::new(),
                    key: Key::TYPE,
                });
                None
            }
        };
        loop {
            if let Some(done) = value.take() {
                match stack.last_mut() {
                    Some(frame) => frame.push(done),
                    None => return done,
                }
            }
            let Some(frame) = stack.last_mut() else {
                return Value::Null;
            };
            if let Some(next) = frame.next_child(store) {
                (key, data) = next;
                break;
            }
            value = stack.pop().map(|frame| frame.finish(store));
        }
    }
}
