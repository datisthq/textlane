use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Link;

use crate::helpers::uri::normalize_uri;
use crate::utils::converter::state::Converter;

/// A link becomes `a`.
pub(crate) fn convert_link<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Link<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let href = converter.alloc.alloc_str(&normalize_uri(node.url));
    let mut properties = ArenaVec::new_in(converter.alloc);
    converter.push_property(&mut properties, "href", Some(href));
    converter.push_property(&mut properties, "title", node.title);
    converter.element_with("a", properties, children, node.position)
}
