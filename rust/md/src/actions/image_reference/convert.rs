use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::ImageReference;

use crate::actions::link_reference::convert::reference_suffix;
use crate::helpers::uri::normalize_uri;
use crate::utils::converter::state::Converter;

/// An image reference becomes `img` from its definition, or its source text when there is
/// none.
pub(crate) fn convert_image_reference<'a>(
    converter: &mut Converter<'a, '_>,
    node: &ImageReference<'a>,
) -> hast::Node<'a> {
    let Some((url, title)) = converter.find_definition(node.identifier) else {
        let suffix = reference_suffix(node.reference_type, node.label, node.identifier);
        // JavaScript writes a missing alt as `null`
        let alt = node.alt.unwrap_or("null");
        let value = converter.concat(&["![", alt, &suffix]);
        return converter.text(value);
    };
    let src = converter.alloc.alloc_str(&normalize_uri(url));
    let mut properties = ArenaVec::new_in(converter.alloc);
    converter.push_property(&mut properties, "src", Some(src));
    converter.push_property(&mut properties, "alt", node.alt);
    converter.push_property(&mut properties, "title", title);
    converter.element_with(
        "img",
        properties,
        ArenaVec::new_in(converter.alloc),
        node.position,
    )
}
