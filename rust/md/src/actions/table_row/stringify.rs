use textlane_core::Error;

use crate::actions::table_cell::stringify::stringify_table_cell;
use crate::helpers::table::format_table;
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Parent, Serializer};

/// A row on its own: the first line of a table of that one row.
pub(crate) fn stringify_table_row(
    state: &mut Serializer,
    row: Parent,
) -> Result<(), Error> {
    let cells = read_table_row(state, row)?;
    let (padding, align_delimiters) = read_table_settings(state);
    let value = format_table(&[cells], &[], padding, align_delimiters);
    state.write(value.split('\n').next().unwrap_or_default());
    Ok(())
}

/// The cells of a row, each serialized.
pub(crate) fn read_table_row(
    state: &mut Serializer,
    row: Parent,
) -> Result<Vec<String>, Error> {
    state.enter(Construct::TableRow);
    let cells = row
        .children
        .iter()
        .map(|cell| stringify_table_cell(state, Parent::of(cell)))
        .collect();
    state.exit();
    cells
}

/// `tableCellPadding` and `tablePipeAlign`.
pub(crate) fn read_table_settings(state: &Serializer) -> (bool, bool) {
    state.settings.gfm.as_ref().map_or((true, true), |gfm| {
        (gfm.table_cell_padding, gfm.table_pipe_align)
    })
}
