use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Strong;

use crate::utils::converter::state::Converter;

/// Strong emphasis becomes `strong`.
pub(crate) fn convert_strong<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Strong<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    converter.element("strong", children, node.position)
}
