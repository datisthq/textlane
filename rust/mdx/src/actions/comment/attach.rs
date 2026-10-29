use crate::models::estree::{Id, Store, Value};
use crate::models::name::{Key, Kind};

const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// estree-util-attach-comments: give each node the comments before it (`leading`) and
/// those after its last child (`trailing`) as its `comments` field, by `start` and `end`.
pub(crate) fn attach_comments(store: &mut Store, tree: Id, comments: &[Id]) {
    let mut list = comments.to_vec();
    list.sort_by(|left, right| order(store, *left, *right));
    if !list.is_empty() {
        let mut index = 0;
        attach(store, tree, &list, &mut index);
    }
}

fn attach(store: &mut Store, node: Id, comments: &[Id], index: &mut usize) {
    if *index == comments.len() {
        return;
    }
    let mut children = Vec::new();
    for (key, value) in store.fields(node) {
        if *key == Key::COMMENTS {
            continue;
        }
        match value {
            Value::Node(child) if !store.is(*child, Kind::NONE) => children.push(*child),
            Value::List(items) => children.extend(
                items
                    .iter()
                    .filter_map(Value::as_node)
                    .filter(|child| !store.is(*child, Kind::NONE)),
            ),
            _ => {}
        }
    }
    children.sort_by(|left, right| order(store, *left, *right));
    let mut attached = slice(store, node, comments, index, Key::START, true, false);
    for child in &children {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
            attach(store, *child, comments, index);
        });
    }
    attached.extend(slice(
        store,
        node,
        comments,
        index,
        Key::END,
        false,
        !children.is_empty(),
    ));
    if !attached.is_empty() {
        store.set(node, Key::COMMENTS, Value::nodes(attached));
    }
}

fn slice(
    store: &mut Store,
    node: Id,
    comments: &[Id],
    index: &mut usize,
    field: Key,
    leading: bool,
    trailing: bool,
) -> Vec<Id> {
    let mut result = Vec::new();
    while let Some(comment) = comments.get(*index)
        && compare(store, *comment, node, field)
            .is_some_and(|difference| difference < 1.0)
    {
        let copy = store.copy(*comment);
        store.set(copy, Key::LEADING, leading);
        store.set(copy, Key::TRAILING, trailing);
        result.push(copy);
        *index += 1;
    }
    result
}

// A comparison upstream cannot make is NaN, which sorts as equal and never slices
fn compare(store: &Store, left: Id, right: Id, field: Key) -> Option<f64> {
    Some(store.number(left, Key::START)? - store.number(right, field)?)
}

// Nodes made by a transform have no `start` and sort last instead
fn order(store: &Store, left: Id, right: Id) -> std::cmp::Ordering {
    let start = |id| store.number(id, Key::START).unwrap_or(f64::INFINITY);
    start(left).total_cmp(&start(right))
}
