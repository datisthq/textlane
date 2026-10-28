use textlane_core::hast::Root;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// The root: wrapped in paragraphs where it mixes phrasing and flow content, or always
/// for a whole `document`.
pub(crate) fn convert_root<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    root: &'n Root<'a>,
) -> Converted {
    let mut children = state.all(&root.children);
    if state.options.document || state.store.is_wrap_needed(&children) {
        children = state.wrap_flow(children);
    }
    Converted::One(state.store.add(Kind::Root, children, root.position))
}
