use textlane_core::hast;
use textlane_core::mdast::InlineMath;

use crate::utils::converter::state::Converter;
use crate::utils::converter::unknown::convert_unknown;

/// Math (text) has no handler upstream: its `data` makes it a `code`.
pub(crate) fn convert_inline_math<'a>(
    converter: &mut Converter<'a, '_>,
    node: &InlineMath<'a>,
) -> hast::Node<'a> {
    convert_unknown(converter, node.value, node.data, node.position)
}
