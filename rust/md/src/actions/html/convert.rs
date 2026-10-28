use textlane_core::hast;
use textlane_core::mdast::Html;

use crate::utils::converter::state::Converter;

/// Raw HTML stays raw when dangerous HTML is allowed, and is dropped otherwise.
pub(crate) fn convert_html<'a>(
    converter: &Converter<'a, '_>,
    node: &Html<'a>,
) -> Option<hast::Node<'a>> {
    converter
        .options
        .allow_dangerous_html
        .then_some(hast::Node::Raw(hast::Raw {
            value: node.value,
            position: node.position,
        }))
}
