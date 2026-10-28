use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::sink::Layer;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a block quote, each line prefixed with `>`.
pub(crate) fn stringify_blockquote(
    state: &mut Serializer,
    blockquote: Parent,
) -> Result<(), Error> {
    state.enter(Construct::Blockquote);
    state.open_layer(Layer::Blockquote);
    let result = state.container_flow(blockquote);
    state.close_layer();
    state.exit();
    result
}
