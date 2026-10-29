use textlane_core::{hast, mdast};

/// An expression or ESM passes into hast as it is, as @mdx-js/mdx has remark-rehype pass
/// MDX nodes through.
pub(crate) fn convert_mdx_expression<'a>(
    node: &mdast::Node<'a>,
) -> Option<hast::Node<'a>> {
    Some(match node {
        mdast::Node::MdxFlowExpression(node) => {
            hast::Node::MdxFlowExpression(node.clone())
        }
        mdast::Node::MdxTextExpression(node) => {
            hast::Node::MdxTextExpression(node.clone())
        }
        mdast::Node::MdxjsEsm(node) => hast::Node::MdxjsEsm(node.clone()),
        _ => return None,
    })
}
