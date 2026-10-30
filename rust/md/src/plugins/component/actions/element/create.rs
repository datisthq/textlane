use textlane_core::mdast::{MdxJsxAttribute, MdxJsxAttributeValue, MdxJsxElement, Node};
use textlane_core::{Allocator, ArenaVec, Data, Position};

/// An MDX JSX element, flow or `text`, with string attributes, marked as explicit JSX as an
/// author's own `<Callout>` is, so the MDX compiler treats both alike.
pub(crate) fn create_element<'a>(
    name: &'a str,
    attributes: Vec<(String, String)>,
    children: ArenaVec<'a, Node<'a>>,
    position: Option<Position<'a>>,
    text: bool,
    alloc: &'a Allocator,
) -> Node<'a> {
    let mut written = ArenaVec::with_capacity_in(attributes.len(), alloc);
    for (name, value) in attributes {
        written.push(MdxJsxAttribute::Attribute {
            name: alloc.alloc_str(&name),
            value: Some(MdxJsxAttributeValue::Literal(alloc.alloc_str(&value))),
            position: None,
            data: None,
        });
    }
    let mut flag = ArenaVec::with_capacity_in(1, alloc);
    flag.push(("_mdxExplicitJsx", Data::Boolean(true)));
    let element = MdxJsxElement {
        name: Some(name),
        attributes: written,
        children,
        position,
        data: Some(alloc.alloc(Data::Object(flag))),
    };
    match text {
        true => Node::MdxJsxTextElement(element),
        false => Node::MdxJsxFlowElement(element),
    }
}
