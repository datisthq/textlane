use textlane_core::hast;
use textlane_core::mdast::Heading;

use crate::utils::converter::state::Converter;

const TAG_NAMES: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];

/// A heading becomes `h1` to `h6`.
pub(crate) fn convert_heading<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Heading<'a>,
) -> hast::Node<'a> {
    let children = converter.all(&node.children);
    let rank = usize::from(node.depth.clamp(1, 6)) - 1;
    converter.element(TAG_NAMES[rank], children, node.position)
}
