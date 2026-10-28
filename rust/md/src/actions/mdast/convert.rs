use textlane_core::Allocator;
use textlane_core::hast;
use textlane_core::mdast;

use crate::models::options::ToHtmlOptions;
use crate::utils::converter::state::Converter;

/// Turn mdast into hast, as remark-rehype does; several nodes come back in a root.
pub fn convert_mdast<'a>(
    node: &mdast::Node<'a>,
    alloc: &'a Allocator,
    options: &ToHtmlOptions,
) -> hast::Node<'a> {
    let mut converter = Converter { alloc, options };
    let mut nodes = converter.convert(node);
    if nodes.len() == 1
        && let Some(node) = nodes.pop()
    {
        return node;
    }
    hast::Node::Root(hast::Root {
        children: nodes,
        position: None,
    })
}
