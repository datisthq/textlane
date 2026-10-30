use textlane_core::mdast::Node;

/// A node's text, as mdast-util-to-string gives it: values, image alts and the text of
/// every child, in order.
pub(crate) fn read_text(node: &Node<'_>) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        match node {
            Node::Text(node) => text.push_str(node.value),
            Node::InlineCode(node) => text.push_str(node.value),
            Node::InlineMath(node) => text.push_str(node.value),
            Node::Html(node) => text.push_str(node.value),
            Node::Code(node) => text.push_str(node.value),
            Node::Math(node) => text.push_str(node.value),
            Node::Literal(node) => text.push_str(node.value),
            Node::Image(node) => text.push_str(node.alt.unwrap_or_default()),
            node => {
                if let Some(children) = node.children() {
                    stack.extend(children.iter().rev());
                }
            }
        }
    }
    text
}
