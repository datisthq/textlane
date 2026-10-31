use memchr::memchr2;
use textlane_core::mdast::{Break, Node, Text};
use textlane_core::{Allocator, ArenaVec};

/// mdast-util-newline-to-break: each line ending in a text becomes a `break`, the text
/// around it new text without a position, as mdast-util-find-and-replace builds it.
pub(crate) fn split_text<'a>(root: &mut Node<'a>, alloc: &'a Allocator) {
    // Like find-and-replace, text that is the whole tree has no parent to be replaced in
    let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
    while let Some(children) = stack.pop() {
        if children.iter().any(has_line_ending) {
            let mut old = std::mem::replace(
                children,
                ArenaVec::with_capacity_in(children.len() + 2, alloc),
            );
            for child in old.drain(..) {
                match child {
                    Node::Text(text) if text.value.contains(['\n', '\r']) => {
                        push_pieces(children, text.value);
                    }
                    child => children.push(child),
                }
            }
        }
        for child in children.iter_mut() {
            if let Some(nested) = child.children_mut() {
                stack.push(nested);
            }
        }
    }
}

fn has_line_ending(node: &Node<'_>) -> bool {
    matches!(node, Node::Text(text) if text.value.contains(['\n', '\r']))
}

// `\r\n` is one line ending, as the expression `/\r?\n|\r/g` reads it
fn push_pieces<'a>(children: &mut ArenaVec<'a, Node<'a>>, value: &'a str) {
    let bytes = value.as_bytes();
    let mut start = 0;
    while let Some(found) = memchr2(b'\n', b'\r', &bytes[start..]) {
        let index = start + found;
        if start < index {
            children.push(text(&value[start..index]));
        }
        children.push(Node::Break(Break {
            position: None,
            data: None,
        }));
        start = if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
            index + 2
        } else {
            index + 1
        };
    }
    if start < value.len() {
        children.push(text(&value[start..]));
    }
}

fn text(value: &str) -> Node<'_> {
    Node::Text(Text {
        value,
        position: None,
        data: None,
    })
}
