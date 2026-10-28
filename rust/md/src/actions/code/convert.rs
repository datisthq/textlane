use textlane_core::hast::{self, Element, Property, PropertyItem, PropertyValue};
use textlane_core::mdast::Code;
use textlane_core::{ArenaString, ArenaVec};

use crate::utils::converter::state::Converter;

/// Code becomes `pre` around `code`, classed by the first word of `lang`.
pub(crate) fn convert_code<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Code<'a>,
) -> hast::Node<'a> {
    let alloc = converter.alloc;
    let value = if node.value.is_empty() {
        ""
    } else {
        let mut value = ArenaString::with_capacity_in(node.value.len() + 1, alloc);
        value.push_str(node.value);
        value.push('\n');
        value.into_bump_str()
    };
    let mut properties = ArenaVec::new_in(alloc);
    if let Some(language) = node.lang.and_then(|lang| lang.split_whitespace().next()) {
        let mut class_name = ArenaString::with_capacity_in(language.len() + 9, alloc);
        class_name.push_str("language-");
        class_name.push_str(language);
        let mut list = ArenaVec::with_capacity_in(1, alloc);
        list.push(PropertyItem::String(class_name.into_bump_str()));
        properties.push(Property {
            name: "className",
            value: PropertyValue::List(list),
        });
    }
    let mut text = ArenaVec::with_capacity_in(1, alloc);
    text.push(hast::Node::Text(hast::Text {
        value,
        position: None,
    }));
    let code = hast::Node::Element(Element {
        tag_name: "code",
        properties,
        children: text,
        position: node.position,
        content: None,
    });
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(code);
    converter.element("pre", children, node.position)
}
