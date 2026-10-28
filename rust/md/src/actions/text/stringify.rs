use textlane_core::mdast::Text;

use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// Write text, escaped against its siblings.
pub(crate) fn stringify_text(
    state: &mut Serializer,
    text: &Text,
    parent: Option<Parent>,
    info: Info,
) {
    let siblings = parent.map_or(&[][..], |parent| parent.children);
    let index = state.index_stack.last().copied();
    let value = state.safe(
        text.value,
        &SafeConfig {
            before: info.before,
            after: info.after,
            before_node: index
                .and_then(|index| index.checked_sub(1))
                .and_then(|index| siblings.get(index)),
            after_node: index.and_then(|index| siblings.get(index + 1)),
            encode: &[],
        },
    );
    state.write(&value);
}
