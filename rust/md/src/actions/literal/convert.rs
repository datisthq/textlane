use textlane_core::hast;
use textlane_core::mdast::Literal;

/// A node of an unknown type with a value becomes text, as mdast-util-to-hast's
/// `unknownHandler` makes it.
pub(crate) fn convert_literal<'a>(node: &Literal<'a>) -> hast::Node<'a> {
    hast::Node::Text(hast::Text {
        value: node.value,
        position: node.position,
        data: None,
    })
}
