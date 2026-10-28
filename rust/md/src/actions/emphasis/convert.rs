use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Emphasis;

use crate::utils::converter::state::Converter;

/// Emphasis becomes `em`.
pub(crate) fn convert_emphasis<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Emphasis<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    converter.element("em", children, node.position)
}
