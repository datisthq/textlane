use textlane_core::mdast::Html;

use crate::utils::serializer::state::Serializer;

/// Write raw HTML as it is.
pub(crate) fn stringify_html(state: &mut Serializer, html: &Html) {
    state.write(html.value);
}
