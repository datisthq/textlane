use textlane_core::Allocator;
use textlane_core::hast;
use textlane_core::mdast;

use crate::models::options::ToHtmlOptions;
use crate::utils::converter::state::Converter;

/// Turn mdast into hast, as remark-rehype does.
pub fn convert_mdast<'a>(
    node: &mdast::Node<'a>,
    alloc: &'a Allocator,
    options: &ToHtmlOptions,
) -> hast::Node<'a> {
    let mut converter = Converter { alloc, options };
    converter.one(node).unwrap_or_else(|| {
        hast::Node::Root(hast::Root {
            children: textlane_core::ArenaVec::new_in(alloc),
            position: None,
        })
    })
}
