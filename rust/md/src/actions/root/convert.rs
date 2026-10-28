use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Root;

use crate::utils::converter::state::Converter;

/// A root stays a root, its children separated by line endings.
pub(crate) fn convert_root<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Root<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    hast::Node::Root(hast::Root {
        children: converter.wrap(children, false),
        position: node.position,
        quirks_mode: None,
    })
}
