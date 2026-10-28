use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// A cell's phrasing, as it goes between the pipes of its row.
pub(crate) fn stringify_table_cell(
    state: &mut Serializer,
    cell: Parent,
) -> Result<String, Error> {
    state.enter(Construct::TableCell);
    state.enter(Construct::Phrasing);
    // Upstream pads with a space only when `tableCellPadding` is passed as `true`, which
    // `GfmOptions` cannot tell from its default
    let value = state.container_phrasing(
        cell,
        Info {
            before: "|",
            after: "|",
        },
    );
    state.exit();
    state.exit();
    value
}
