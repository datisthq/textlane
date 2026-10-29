use crate::models::estree::{Id, Store, Value};

/// What `enter` asks of a walk, as estree-walker's `this.skip()` and `this.replace()`.
#[derive(Default)]
pub(crate) struct Enter {
    pub(crate) skip: bool,
    pub(crate) replace: Option<Id>,
}

/// A visitor in estree-walker's style: `enter` before a node's children, `leave` after,
/// either may replace the node in its parent.
pub(crate) trait Visit {
    fn enter(&mut self, store: &mut Store, id: Id) -> Enter;
    fn leave(&mut self, store: &mut Store, id: Id) -> Option<Id>;
}

struct Frame {
    id: Id,
    place: Option<(Id, usize, Option<usize>)>,
    field: usize,
    item: usize,
    fields: usize,
}

/// Walk `root` depth first as estree-walker does: every field holding an object with a
/// type, or a list of them, in field order; a skipped node is not left. Gives the root,
/// replaced or not.
pub(crate) fn walk(store: &mut Store, root: Id, visitor: &mut impl Visit) -> Id {
    let mut root = root;
    let mut stack: Vec<Frame> = Vec::new();
    let mut next = Some((root, None));
    loop {
        if let Some((id, place)) = next.take() {
            let enter = visitor.enter(store, id);
            let mut id = id;
            if let Some(replacement) = enter.replace {
                put(store, &place, replacement, &mut root);
                id = replacement;
            }
            if !enter.skip {
                let fields = store.node(id).fields.len();
                stack.push(Frame {
                    id,
                    place,
                    field: 0,
                    item: 0,
                    fields,
                });
            }
        }
        let Some(frame) = stack.last_mut() else {
            return root;
        };
        while frame.field < frame.fields {
            let (_, value) = &store.node(frame.id).fields[frame.field];
            let found = match value {
                Value::Node(child) if frame.item == 0 && is_node(store, *child) => {
                    Some((*child, None))
                }
                Value::List(items) if frame.item < items.len() => {
                    let index = frame.item;
                    match items[index] {
                        Value::Node(child) if is_node(store, child) => {
                            Some((child, Some(index)))
                        }
                        _ => {
                            frame.item += 1;
                            continue;
                        }
                    }
                }
                _ => None,
            };
            match found {
                Some((child, index)) => {
                    next = Some((child, Some((frame.id, frame.field, index))));
                    frame.item += 1;
                    break;
                }
                None => {
                    frame.field += 1;
                    frame.item = 0;
                }
            }
        }
        if next.is_none()
            && let Some(frame) = stack.pop()
            && let Some(replacement) = visitor.leave(store, frame.id)
        {
            put(store, &frame.place, replacement, &mut root);
        }
    }
}

fn is_node(store: &Store, id: Id) -> bool {
    !store.kind(id).is_empty()
}

fn put(
    store: &mut Store,
    place: &Option<(Id, usize, Option<usize>)>,
    replacement: Id,
    root: &mut Id,
) {
    match place {
        Some((parent, field, Some(index))) => {
            if let Some(Value::List(items)) = store.value_at_mut(*parent, *field)
                && let Some(item) = items.get_mut(*index)
            {
                *item = Value::Node(replacement);
            }
        }
        Some((parent, field, None)) => {
            if let Some(value) = store.value_at_mut(*parent, *field) {
                *value = Value::Node(replacement);
            }
        }
        None => *root = replacement,
    }
}
