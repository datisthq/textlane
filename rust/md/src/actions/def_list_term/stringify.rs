use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// Write a definition term on a line of its own.
pub(crate) fn stringify_def_list_term(
    state: &mut Serializer,
    term: Parent,
) -> Result<(), Error> {
    state.enter(Construct::DefListTerm);
    state.enter(Construct::Phrasing);
    let info = Info {
        before: "\n",
        after: "\n",
    };
    let value = state.container_phrasing(term, info);
    state.exit();
    state.exit();
    state.write(&value?);
    Ok(())
}
