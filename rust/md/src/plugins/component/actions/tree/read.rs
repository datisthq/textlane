use serde_json::{Map, Value, json};
use textlane_core::mdast::{ListItem, Node};

use crate::plugins::component::actions::text::read::read_text;

/// The file tree the lists among `children` describe, as Starlight's `<FileTree>` reads a
/// list: a name ending in `/` or holding a list is a directory, a bold name is highlighted,
/// text after the name is its comment and `…` or `...` alone stands for more files.
pub(crate) fn read_tree(children: &[Node<'_>]) -> Value {
    Value::Array(read_lists(children, 0))
}

// serde_json writes and drops a value by recursion, so a tree deeper than any real one
// stops here rather than overflowing whoever serializes it
const MAX_DEPTH: usize = 100;

fn read_lists(children: &[Node<'_>], depth: usize) -> Vec<Value> {
    if depth >= MAX_DEPTH {
        return Vec::new();
    }
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        let items = children.iter().filter_map(|child| match child {
            Node::List(list) => Some(list.children.iter()),
            _ => None,
        });
        items
            .flatten()
            .filter_map(|item| match item {
                Node::ListItem(item) => read_item(item, depth),
                _ => None,
            })
            .collect()
    })
}

// An item's name, type and comment come from its first paragraph; one without a
// paragraph or a name is left out, as livemark leaves it out
fn read_item(item: &ListItem<'_>, depth: usize) -> Option<Value> {
    let paragraph = item.children.iter().find_map(|child| match child {
        Node::Paragraph(paragraph) => Some(paragraph),
        _ => None,
    })?;
    let inline = &paragraph.children;
    let (name, comment, highlighted) = match inline.first() {
        Some(Node::Strong(strong)) => {
            let rest: String = inline.iter().skip(1).map(read_text).collect();
            (read_text_of(&strong.children), rest, true)
        }
        Some(Node::InlineCode(code)) => {
            let rest: String = inline.iter().skip(1).map(read_text).collect();
            (code.value.to_owned(), rest, false)
        }
        _ => {
            let text: String = inline.iter().map(read_text).collect();
            let text = text.trim();
            match text.split_once(char::is_whitespace) {
                Some((name, rest)) => (name.to_owned(), rest.to_owned(), false),
                None => (text.to_owned(), String::new(), false),
            }
        }
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let children = read_lists(&item.children, depth + 1);
    let nested = item
        .children
        .iter()
        .any(|child| matches!(child, Node::List(_)));
    let (name, kind) = match name.strip_suffix('/') {
        _ if name == "…" || name == "..." => (name, "placeholder"),
        Some(stripped) if !stripped.is_empty() => (stripped, "directory"),
        _ if nested => (name, "directory"),
        _ => (name, "file"),
    };
    let mut entry = Map::new();
    entry.insert("name".into(), json!(name));
    entry.insert("type".into(), json!(kind));
    let comment = comment.trim();
    if !comment.is_empty() {
        entry.insert("comment".into(), json!(comment));
    }
    if highlighted {
        entry.insert("highlighted".into(), json!(true));
    }
    if !children.is_empty() {
        entry.insert("children".into(), Value::Array(children));
    }
    Some(Value::Object(entry))
}

fn read_text_of(children: &[Node<'_>]) -> String {
    children.iter().map(read_text).collect()
}
