use textlane_core::Data;
use textlane_core::mdast::Node;

/// mdast-util-to-string with `includeImageAlt: false`: the values of `nodes` and of their
/// descendants, in order.
pub(crate) fn collect_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    let mut stack = vec![nodes.iter()];
    while let Some(siblings) = stack.last_mut() {
        let Some(node) = siblings.next() else {
            stack.pop();
            continue;
        };
        if let Some(value) = read_value(node) {
            text.push_str(value);
        } else if let Some(children) = node.children() {
            stack.push(children.iter());
        }
    }
    text
}

// A node with a `value` field gives it and never its children
fn read_value<'a>(node: &Node<'a>) -> Option<&'a str> {
    match node {
        Node::Text(node) => Some(node.value),
        Node::InlineCode(node) => Some(node.value),
        Node::Html(node) => Some(node.value),
        Node::Code(node) => Some(node.value),
        Node::Math(node) => Some(node.value),
        Node::InlineMath(node) => Some(node.value),
        Node::Yaml(node) => Some(node.value),
        Node::Toml(node) => Some(node.value),
        Node::Literal(node) => Some(node.value),
        Node::MdxFlowExpression(node) | Node::MdxTextExpression(node) => Some(node.value),
        Node::MdxjsEsm(node) => Some(node.value),
        Node::Unknown(node) => node
            .fields
            .iter()
            .rev()
            .find(|(key, _)| *key == "value")
            .map(|(_, value)| match value {
                Data::String(value) => *value,
                _ => "",
            }),
        _ => None,
    }
}
