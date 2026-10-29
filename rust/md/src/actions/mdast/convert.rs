use std::collections::HashMap;

use textlane_core::Allocator;
use textlane_core::hast;
use textlane_core::mdast;

use crate::models::options::ToHtmlOptions;
use crate::utils::converter::footnote::{
    append_footer, create_footer, find_footnote_definitions,
};
use crate::utils::converter::state::Converter;

/// Turn mdast into hast, as remark-rehype does; several nodes come back in a root.
pub fn convert_mdast<'a>(
    node: &mdast::Node<'a>,
    alloc: &'a Allocator,
    options: &ToHtmlOptions,
) -> hast::Node<'a> {
    convert_mdast_with(node, alloc, options, false)
}

/// `convert_mdast`, lifting MDX out of paragraphs first with `unravel`, as @mdx-js/mdx's
/// remark-mark-and-unravel does.
pub(crate) fn convert_mdast_with<'a>(
    node: &mdast::Node<'a>,
    alloc: &'a Allocator,
    options: &ToHtmlOptions,
    unravel: bool,
) -> hast::Node<'a> {
    let mut converter = Converter {
        alloc,
        options,
        unravel,
        loose_lists: vec![],
        definitions: find_definitions(node),
        footnote_order: vec![],
        footnote_counts: HashMap::new(),
    };
    let mut nodes = converter.convert(node);
    // Handlers that return several nodes, or text in place of a reference, return a list,
    // which mdast-util-to-hast wraps in a root
    let list = match node {
        mdast::Node::Break(_) => true,
        mdast::Node::LinkReference(reference) => {
            converter.find_definition(reference.identifier).is_none()
        }
        mdast::Node::ImageReference(reference) => {
            converter.find_definition(reference.identifier).is_none()
        }
        _ => false,
    };
    let mut tree = match nodes.pop() {
        Some(node) if nodes.is_empty() && !list => node,
        last => {
            nodes.extend(last);
            hast::Node::Root(hast::Root {
                children: nodes,
                position: None,
                quirks_mode: None,
                data: None,
            })
        }
    };
    let footnotes = find_footnote_definitions(node);
    if let Some(footer) = create_footer(&mut converter, &footnotes) {
        append_footer(&mut tree, footer);
    }
    tree
}

// Every definition in the tree, walked without recursing; the first of an identifier wins
fn find_definitions<'a>(
    node: &mdast::Node<'a>,
) -> HashMap<String, (&'a str, Option<&'a str>)> {
    let mut definitions = HashMap::new();
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        if let mdast::Node::Definition(definition) = node {
            definitions
                .entry(definition.identifier.to_uppercase())
                .or_insert((definition.url, definition.title));
        }
        if let Some(children) = node.children() {
            stack.extend(children.iter().rev());
        }
    }
    definitions
}
