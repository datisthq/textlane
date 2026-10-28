use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Break;

use crate::utils::converter::state::Converter;

/// A hard break becomes `br`; the converter adds the line ending after it.
pub(crate) fn convert_hard_break<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Break<'a>,
) -> hast::Node<'a> {
    converter.element("br", ArenaVec::new_in(converter.alloc), node.position)
}
