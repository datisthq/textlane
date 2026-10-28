use textlane_core::hast;
use textlane_core::mdast::Paragraph;

use crate::utils::converter::state::Converter;

/// A paragraph becomes `p`.
pub(crate) fn convert_paragraph<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Paragraph<'a>,
) -> hast::Node<'a> {
    let children = converter.all(&node.children);
    converter.element("p", children, node.position)
}
