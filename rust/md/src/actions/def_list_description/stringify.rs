use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::sink::Layer;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a definition description: `:` and three spaces, its content indented by four.
pub(crate) fn stringify_def_list_description(
    state: &mut Serializer,
    description: Parent,
) -> Result<(), Error> {
    state.enter(Construct::DefListDescription);
    state.open_layer(Layer::DefListDescription);
    let result = state.container_flow(description);
    state.close_layer();
    state.exit();
    result
}
