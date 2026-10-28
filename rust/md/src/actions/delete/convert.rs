use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Delete;

use crate::utils::converter::state::Converter;

/// A delete becomes `del`.
pub(crate) fn convert_delete<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Delete<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    converter.element("del", children, node.position)
}
