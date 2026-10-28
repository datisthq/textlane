use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a definition list: its terms and descriptions, joined as `join_def_list` says.
pub(crate) fn stringify_def_list(
    state: &mut Serializer,
    list: Parent,
) -> Result<(), Error> {
    state.enter(Construct::DefList);
    let result = state.container_flow(list);
    state.exit();
    result
}
