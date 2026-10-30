use textlane_core::Data;
use textlane_core::nlcst::Node;

/// The text of an nlcst node: its value, or its children's joined, as nlcst-to-string.
pub fn stringify_nlcst(node: &Node<'_>) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        if let Some(value) = read_value(node) {
            text.push_str(value);
        } else if let Some(children) = node.children() {
            stack.extend(children.iter().rev());
        }
    }
    text
}

/// A node's `value`, which a node of an unknown type may carry too.
pub(crate) fn read_value<'a>(node: &Node<'a>) -> Option<&'a str> {
    match node {
        Node::Unknown(unknown) => {
            unknown.fields.iter().find_map(|(key, value)| match value {
                Data::String(value) if *key == "value" => Some(*value),
                _ => None,
            })
        }
        _ => node.value(),
    }
}
