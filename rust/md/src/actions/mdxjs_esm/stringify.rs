use textlane_core::mdast::MdxjsEsm;

use crate::utils::serializer::state::Serializer;

/// Write ESM as its source.
pub(crate) fn stringify_mdxjs_esm(state: &mut Serializer, esm: &MdxjsEsm) {
    state.write(esm.value);
}
