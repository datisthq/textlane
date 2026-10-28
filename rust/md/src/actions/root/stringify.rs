use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::is_phrasing;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// Write a root: as phrasing when any child is phrasing, as flow otherwise.
pub(crate) fn stringify_root(
    state: &mut Serializer,
    root: Parent,
    info: Info,
) -> Result<(), Error> {
    if !root.children.iter().any(is_phrasing) {
        return state.container_flow(root);
    }
    state.enter(Construct::Phrasing);
    let value = state.container_phrasing(root, info);
    state.exit();
    state.write(&value?);
    Ok(())
}
