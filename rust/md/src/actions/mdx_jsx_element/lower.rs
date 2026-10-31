use textlane_core::hast::{self, Property, PropertyValue};
use textlane_core::mdast::{self, MdxJsxAttribute, MdxJsxAttributeValue};
use textlane_core::{Allocator, ArenaVec};

/// A JSX element as the plain HTML element it names, for `md.toHtml`'s `jsxElements`:
/// literal attributes as properties, a bare or empty one `true`, expressions dropped; a
/// fragment, `<>`, gives way to its children.
pub(crate) fn lower_mdx_jsx_element<'a>(
    node: &mdast::Node<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
    alloc: &'a Allocator,
) -> ArenaVec<'a, hast::Node<'a>> {
    let element = match node {
        mdast::Node::MdxJsxFlowElement(element)
        | mdast::Node::MdxJsxTextElement(element) => element,
        _ => return ArenaVec::new_in(alloc),
    };
    let Some(name) = element.name else {
        return children;
    };
    let mut properties = ArenaVec::with_capacity_in(element.attributes.len(), alloc);
    for attribute in element.attributes.iter() {
        let MdxJsxAttribute::Attribute { name, value, .. } = attribute else {
            continue;
        };
        // hast reads `""` on a boolean attribute (`open`) as false, and `a` is `a=""` anyway
        let value = match value {
            None | Some(MdxJsxAttributeValue::Literal("")) => {
                PropertyValue::Boolean(true)
            }
            Some(MdxJsxAttributeValue::Literal(value)) => PropertyValue::String(value),
            Some(MdxJsxAttributeValue::Expression { .. }) => continue,
        };
        properties.push(Property { name, value });
    }
    let mut lowered = ArenaVec::with_capacity_in(1, alloc);
    lowered.push(hast::Node::Element(hast::Element {
        tag_name: name,
        properties,
        children,
        position: element.position,
        data: None,
        content: None,
    }));
    lowered
}
