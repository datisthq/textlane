use textlane_core::{ArenaVec, hast, mdast};

/// A JSX element passes into hast as it is, its children converted, as @mdx-js/mdx has
/// remark-rehype pass MDX nodes through.
pub(crate) fn convert_mdx_jsx_element<'a>(
    node: &mdast::Node<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> Option<hast::Node<'a>> {
    let (element, flow) = match node {
        mdast::Node::MdxJsxFlowElement(element) => (element, true),
        mdast::Node::MdxJsxTextElement(element) => (element, false),
        _ => return None,
    };
    let mut attributes =
        ArenaVec::with_capacity_in(element.attributes.len(), element.attributes.bump());
    attributes.extend(element.attributes.iter().cloned());
    let element = hast::MdxJsxElement {
        name: element.name,
        attributes,
        children,
        position: element.position,
        data: element.data,
    };
    Some(if flow {
        hast::Node::MdxJsxFlowElement(element)
    } else {
        hast::Node::MdxJsxTextElement(element)
    })
}
