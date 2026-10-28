use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::TableCell;

use crate::utils::converter::state::Converter;

/// A cell becomes `td`; its row makes it `th` in a head.
pub(crate) fn convert_table_cell<'a>(
    converter: &mut Converter<'a, '_>,
    node: &TableCell<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    converter.element("td", children, node.position)
}
