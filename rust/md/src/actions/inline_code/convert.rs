use textlane_core::hast;
use textlane_core::mdast::InlineCode;
use textlane_core::{ArenaString, ArenaVec};

use crate::utils::converter::state::Converter;

/// A code span becomes `code`, its line endings spaces.
pub(crate) fn convert_inline_code<'a>(
    converter: &mut Converter<'a, '_>,
    node: &InlineCode<'a>,
) -> hast::Node<'a> {
    let value = if node.value.contains(['\n', '\r']) {
        let mut value = ArenaString::with_capacity_in(node.value.len(), converter.alloc);
        value.push_str(&node.value.replace("\r\n", " ").replace(['\n', '\r'], " "));
        value.into_bump_str()
    } else {
        node.value
    };
    let mut children = ArenaVec::with_capacity_in(1, converter.alloc);
    children.push(hast::Node::Text(hast::Text {
        value,
        position: None,
    }));
    converter.element("code", children, node.position)
}
