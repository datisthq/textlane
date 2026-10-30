use serde_json::{Map, Value, json};
use textlane_core::hast::{Node, PropertyValue};

use crate::helpers::property::find_property;
use crate::helpers::schema::HTML;

/// The headings of a tree as @stefanprobst/rehype-extract-toc nests them: each
/// `{depth, value, id?, children?}`, under the closest heading of a lower rank before it.
pub fn extract_toc(node: &Node<'_>, skip: &[String]) -> Value {
    let skipped: Vec<String> = skip.iter().map(|name| read_attribute(name)).collect();
    let mut headings = Vec::new();
    let mut stack: Vec<&Node<'_>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &root.children,
            Node::Element(element) => {
                let left_out = element.properties.iter().any(|property| {
                    !matches!(
                        property.value,
                        PropertyValue::Null | PropertyValue::Boolean(false)
                    ) && skipped.contains(&read_attribute(property.name))
                });
                if let Some(depth) = read_rank(element.tag_name).filter(|_| !left_out) {
                    let mut heading = Map::new();
                    heading.insert("depth".into(), json!(depth));
                    heading.insert("value".into(), json!(read_text(node)));
                    if let Some(id) = read_id(element) {
                        heading.insert("id".into(), id);
                    }
                    headings.push((depth, heading));
                }
                &element.children
            }
            _ => continue,
        };
        stack.extend(children.iter().rev());
    }
    nest_headings(headings)
}

// `createTree`: a heading deeper than the one before opens it as a parent; a shallower one
// closes parents down to one shallower than itself
fn nest_headings(headings: Vec<(u8, Map<String, Value>)>) -> Value {
    let mut parents: Vec<(u8, Map<String, Value>, Vec<Value>)> =
        vec![(0, Map::new(), vec![])];
    let mut previous_depth = 0;
    for (depth, heading) in headings {
        if depth > previous_depth {
            // the previous heading becomes a parent once something nests in it
        } else {
            while parents.len() > 1 && parents.last().is_some_and(|(d, _, _)| *d >= depth)
            {
                close_parent(&mut parents);
            }
        }
        parents.push((depth, heading, vec![]));
        previous_depth = depth;
    }
    while parents.len() > 1 {
        close_parent(&mut parents);
    }
    let root = parents
        .pop()
        .map(|(_, _, children)| children)
        .unwrap_or_default();
    Value::Array(root)
}

// The innermost open heading joins its parent's children, with its own when it has any
fn close_parent(parents: &mut Vec<(u8, Map<String, Value>, Vec<Value>)>) {
    let Some((_, mut heading, children)) = parents.pop() else {
        return;
    };
    if !children.is_empty() {
        heading.insert("children".into(), Value::Array(children));
    }
    if let Some((_, _, siblings)) = parents.last_mut() {
        siblings.push(Value::Object(heading));
    }
}

// hast-util-heading-rank
fn read_rank(tag_name: &str) -> Option<u8> {
    match tag_name.to_ascii_lowercase().as_bytes() {
        [b'h', digit @ b'1'..=b'6'] => Some(digit - b'0'),
        _ => None,
    }
}

// hast-util-to-string: the text of every descendant, in order
fn read_text(node: &Node<'_>) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        match node {
            Node::Text(node) => text.push_str(node.value),
            Node::Element(element) => stack.extend(element.children.iter().rev()),
            Node::Root(root) => stack.extend(root.children.iter().rev()),
            _ => {}
        }
    }
    text
}

fn read_id(element: &textlane_core::hast::Element<'_>) -> Option<Value> {
    let property = element
        .properties
        .iter()
        .find(|property| property.name == "id")?;
    match &property.value {
        PropertyValue::String(id) => Some(json!(id)),
        PropertyValue::Number(id) => Some(json!(id)),
        PropertyValue::Boolean(id) => Some(json!(id)),
        PropertyValue::List(_) | PropertyValue::Null => None,
    }
}

// The HTML attribute a property or attribute name stands for, `dataStep` and `data-step` alike
fn read_attribute(name: &str) -> String {
    find_property(&HTML, name).attribute.to_lowercase()
}
