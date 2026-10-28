use textlane_core::hast;
use textlane_core::mdast::Math;

use crate::utils::converter::state::Converter;
use crate::utils::converter::unknown::convert_unknown;

/// Math (flow) has no handler upstream: its `data` makes it a `pre` and a `code`.
pub(crate) fn convert_math<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Math<'a>,
) -> hast::Node<'a> {
    convert_unknown(converter, node.value, node.data, node.position)
}
