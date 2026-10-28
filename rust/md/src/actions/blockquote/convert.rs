use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Blockquote;

use crate::utils::converter::state::Converter;

/// A block quote becomes `blockquote`, its children on lines of their own.
pub(crate) fn convert_blockquote<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Blockquote<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let children = converter.wrap(children, true);
    converter.element("blockquote", children, node.position)
}
