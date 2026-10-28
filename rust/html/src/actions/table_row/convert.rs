use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<tr>`: a table row, its content in cells.
pub(crate) fn convert_table_row<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let children = state.all(&element.children);
    let children = state.wrap_specific_content(children, Kind::TableCell(None));
    Converted::One(state.store.add(Kind::TableRow, children, element.position))
}
