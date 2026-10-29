use crate::models::estree::{Id, Store, Value};

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
    for (key, value) in &store.node(node).fields {
        if &**key == "comments" {
            continue;
        }
        match value {
            Value::Node(child) if !store.kind(*child).is_empty() => children.push(*child),
            Value::List(items) => children.extend(
                items
                    .iter()
                    .filter_map(Value::as_node)
                    .filter(|child| !store.kind(*child).is_empty()),
            ),
            _ => {}
        }
    }
    children.sort_by(|left, right| order(store, *left, *right));
    let mut attached = slice(store, node, comments, index, "start", true, false);
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
        "end",
        false,
        !children.is_empty(),
    ));
    if !attached.is_empty() {
        store.set(node, "comments", Value::nodes(attached));
    }
}

fn slice(
    store: &mut Store,
    node: Id,
    comments: &[Id],
    index: &mut usize,
    field: &str,
    leading: bool,
    trailing: bool,
) -> Vec<Id> {
    let mut result = Vec::new();
    while let Some(comment) = comments.get(*index)
        && compare(store, *comment, node, field)
            .is_some_and(|difference| difference < 1.0)
    {
        let copy = store.copy(*comment);
        store.set(copy, "leading", leading);
        store.set(copy, "trailing", trailing);
        result.push(copy);
        *index += 1;
    }
    result
}

// A comparison upstream cannot make is NaN, which sorts as equal and never slices
fn compare(store: &Store, left: Id, right: Id, field: &str) -> Option<f64> {
    Some(store.number(left, "start")? - store.number(right, field)?)
}

// Nodes made by a transform have no `start` and sort last instead
fn order(store: &Store, left: Id, right: Id) -> std::cmp::Ordering {
    let start = |id| store.number(id, "start").unwrap_or(f64::INFINITY);
    start(left).total_cmp(&start(right))
}
