use textlane_core::Error;
use textlane_core::mdast::Table;

use crate::actions::table_row::stringify::{read_table_row, read_table_settings};
use crate::helpers::table::format_table;
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a table: its rows, the delimiter row after the first.
pub(crate) fn stringify_table(
    state: &mut Serializer,
    table: &Table,
    node: Parent,
) -> Result<(), Error> {
    state.enter(Construct::Table);
    let rows = node
        .children
        .iter()
        .map(|row| read_table_row(state, Parent::of(row)))
        .collect::<Result<Vec<_>, Error>>();
    state.exit();
    let (padding, align_delimiters) = read_table_settings(state);
    let value = format_table(&rows?, &table.align, padding, align_delimiters);
    state.write(&value);
    Ok(())
}
