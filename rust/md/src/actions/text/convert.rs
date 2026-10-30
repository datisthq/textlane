use std::borrow::Cow;

use textlane_core::hast;
use textlane_core::mdast::Text;

use crate::helpers::lines::trim_lines;
use crate::utils::converter::state::Converter;

/// Text stays text, without the spaces and tabs around its line endings.
pub(crate) fn convert_text<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Text<'a>,
) -> hast::Node<'a> {
    let value = match trim_lines(node.value) {
        Cow::Borrowed(_) => node.value,
        Cow::Owned(value) => converter.alloc.alloc_str(&value),
    };
    hast::Node::Text(hast::Text {
        value,
        position: node.position,
        data: None,
    })
}
