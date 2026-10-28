use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::Image;

use crate::helpers::uri::normalize_uri;
use crate::utils::converter::state::Converter;

/// An image becomes `img`.
pub(crate) fn convert_image<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Image<'a>,
) -> hast::Node<'a> {
    let src = converter.alloc.alloc_str(&normalize_uri(node.url));
    let mut properties = ArenaVec::new_in(converter.alloc);
    converter.push_property(&mut properties, "src", Some(src));
    converter.push_property(&mut properties, "alt", node.alt);
    converter.push_property(&mut properties, "title", node.title);
    converter.element_with(
        "img",
        properties,
        ArenaVec::new_in(converter.alloc),
        node.position,
    )
}
