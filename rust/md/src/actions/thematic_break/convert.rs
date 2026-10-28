use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::ThematicBreak;

use crate::utils::converter::state::Converter;

/// A thematic break becomes `hr`.
pub(crate) fn convert_thematic_break<'a>(
    converter: &mut Converter<'a, '_>,
    node: &ThematicBreak<'a>,
) -> hast::Node<'a> {
    converter.element("hr", ArenaVec::new_in(converter.alloc), node.position)
}
