use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::DefListTerm;

use crate::utils::converter::state::Converter;

/// A definition term becomes `dt`.
pub(crate) fn convert_def_list_term<'a>(
    converter: &mut Converter<'a, '_>,
    node: &DefListTerm<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    converter.element("dt", children, node.position)
}
